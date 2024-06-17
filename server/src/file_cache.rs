use crate::config::CacheConfig;
use crate::Config;
use anyhow::anyhow;
use dashmap::DashMap;
use flate2::read::GzEncoder;
use flate2::Compression;
use hyper::body::Bytes;
use lazy_static::lazy_static;
use mime::Mime;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::{File, Metadata};
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use walkdir::WalkDir;
use warp::fs::ArcPath;

pub struct FileCache {
    conf: DomainCacheConfig,
    data: DashMap<String, HashMap<String, Arc<CacheItem>>>,
}

pub struct DomainCacheConfig {
    default: CacheConfig,
    inner: HashMap<String, CacheConfig>,
}

impl DomainCacheConfig {
    pub fn new(conf: &Config) -> Self {
        let default = conf.cache.clone();
        let inner: HashMap<String, CacheConfig> = conf
            .domains
            .iter()
            .map(|domain| {
                let cache = domain.cache.as_ref();
                let max_size = cache
                    .map(|x| x.max_size)
                    .flatten()
                    .unwrap_or(default.max_size);
                let compression = cache
                    .map(|x| x.compression)
                    .flatten()
                    .unwrap_or(default.compression);
                let client_cache = cache
                    .map(|x| x.client_cache.as_ref())
                    .flatten()
                    .unwrap_or(&default.client_cache)
                    .clone();
                (
                    domain.domain.clone(),
                    CacheConfig {
                        max_size,
                        compression,
                        client_cache,
                    },
                )
            })
            .collect();

        DomainCacheConfig { default, inner }
    }
    pub fn get_domain_cache_config(&self, domain: &str) -> &CacheConfig {
        self.inner.get(domain).unwrap_or(&self.default)
    }
    pub fn get_domain_expire_config(&self, domain: &str) -> HashMap<String, Duration> {
        let conf = self.get_domain_cache_config(domain);
        let ret: HashMap<String, Duration> = conf
            .client_cache
            .iter()
            .map(|item| {
                item.extension_names
                    .iter()
                    .map(|extension_name| (extension_name.clone(), item.expire.clone()))
                    .collect::<Vec<(String, Duration)>>()
            })
            .flatten()
            .collect();
        ret
    }
}

lazy_static! {
    pub static ref COMPRESSION_FILE_TYPE: HashSet<String> = HashSet::from([
        String::from("html"),
        String::from("js"),
        String::from("icon"),
        String::from("json"),
        String::from("css")
    ]);
}

impl FileCache {
    pub fn new(conf: &Config) -> Self {
        let conf = DomainCacheConfig::new(conf);
        FileCache {
            conf,
            data: DashMap::new(),
        }
    }

    pub fn update(
        &self,
        domain: String,
        sub_path: Option<&str>,
        version: u32,
        data: HashMap<String, Arc<CacheItem>>,
    ) {
        let data = match self.data.get(&domain) {
            Some(ref info) => {
                let mut new_hash_map: HashMap<String, Arc<CacheItem>> = info
                    .value()
                    .iter()
                    .filter_map(|(v, k)| match sub_path {
                        Some(sub_path) => {
                            if v.starts_with(sub_path)
                                && version > k.version
                                && version - k.version > 2
                            {
                                None
                            } else {
                                Some((v.clone(), k.clone()))
                            }
                        }
                        None => {
                            if version > k.version && version - k.version > 2 {
                                None
                            } else {
                                Some((v.clone(), k.clone()))
                            }
                        }
                    })
                    .collect();
                // inesrt before get would trigger deadlock. so move out insert function
                //drop(info);
                new_hash_map.extend(data);
                new_hash_map
            }
            None => data,
        };

        self.data.insert(domain, data);
    }

    pub fn cache_dir(
        &self,
        domain: &str, //www.example.com
        sub_path: Option<&str>,
        version: u32,
        path: &PathBuf,
    ) -> anyhow::Result<HashMap<String, Arc<CacheItem>>> {
        let prefix = path
            .to_str()
            .map(|x| Ok(format!("{}/", x.to_string())))
            .unwrap_or(Err(anyhow!("can not parse path")))?;
        tracing::info!("prepare to cache_dir: {}", &prefix);
        let conf = self.conf.get_domain_cache_config(domain);
        let expire_config = self.conf.get_domain_expire_config(domain);
        let result: HashMap<String, Arc<CacheItem>> = WalkDir::new(path)
            .min_depth(1)
            .into_iter()
            .filter_map(|x| x.ok())
            .filter_map(|entry| {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        let file = File::open(path).ok()?;
                        let mut reader = BufReader::new(file);
                        let mut bytes: Vec<u8> = vec![];
                        reader.read_to_end(&mut bytes).ok()?;
                        let mime = mime_guess::from_path(path).first_or_octet_stream();
                        let entry_path = entry.into_path();

                        return entry_path.clone().to_str().map(|x| {
                            let key = x.replace(&prefix, "");
                            let extension_name = key
                                .split('.')
                                .last()
                                .map_or("".to_string(), |x| x.to_string());

                            let key = sub_path
                                .map(|sub_path| format!("{sub_path}/{key}"))
                                .unwrap_or(key);

                            let data_block = if conf.max_size < metadata.len() {
                                tracing::debug!(
                                    "file block:{}, url:{domain}/{key}",
                                    entry_path.display()
                                );
                                DataBlock::FileBlock(ArcPath(Arc::new(entry_path)))
                            } else {
                                let (bytes, compressed) = if conf.compression
                                    && COMPRESSION_FILE_TYPE.contains(&extension_name)
                                {
                                    let mut encoded_bytes = Vec::new();
                                    let mut encoder =
                                        GzEncoder::new(&bytes[..], Compression::default());
                                    encoder.read_to_end(&mut encoded_bytes).unwrap();

                                    (Bytes::from(encoded_bytes), true)
                                } else {
                                    (Bytes::from(bytes), false)
                                };
                                tracing::debug!(
                                    "cache block:{:?}, compressed:{compressed}, url:{domain}/{key}",
                                    entry_path,
                                );

                                DataBlock::CacheBlock {
                                    bytes,
                                    compressed,
                                    path: ArcPath(Arc::new(entry_path)),
                                }
                            };
                            //let e_tag = etag_calculate(&metadata, real_len).ok()?;
                            (
                                key,
                                Arc::new(CacheItem {
                                    mime,
                                    meta: metadata,
                                    data: data_block,
                                    version,
                                    expire: expire_config.get(&extension_name).cloned(),
                                }),
                            )
                        });
                    }
                }
                None
            })
            .collect();
        Ok(result)
    }

    pub fn get_item(&self, host: &str, path: &str) -> Option<Arc<CacheItem>> {
        self.data
            .get(host)
            .map(|x| {
                return x.get(path).map(Arc::clone);
            })
            .flatten()
    }
    pub fn get_all_keys(&self, host: &str) -> Vec<String> {
        self.data
            .get(host)
            .map(|x| {
                let keys = x.value().keys();
                keys.map(|x| x.to_string()).collect()
            })
            .unwrap_or_else(|| vec![])
    }
    pub fn delete_by_host(&self, host:&str, sub_dir:Option<String>, version:Option<u32>) {
        match (sub_dir, version) {
            (None,None) => {
                self.data.remove(host);
            }
            (sub_dir, version) => {
                let map = self.data.get(host).map(|x| {
                   x.iter().filter_map(|(key, value)| {
                       if sub_dir.as_ref().map(|sub_dir| key.starts_with(sub_dir)).unwrap_or(true) &&
                           version.as_ref().map(|version| value.version == *version).unwrap_or(true)
                           {
                           None
                       }else {
                           Some((key.clone(), value.clone()))
                       }
                   }).collect::<HashMap<String, Arc<CacheItem>>>()
                });
                if let Some(keys) = map {
                    self.data.insert(host.to_string(), keys);
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum DataBlock {
    CacheBlock {
        bytes: Bytes,
        compressed: bool,
        path: ArcPath,
    },
    // for use warp
    FileBlock(ArcPath),
}
/*
impl DataBlock {
    pub fn get_path(&self) -> &ArcPath {
        match self {
            DataBlock::CacheBlock {path,..} => path,
            DataBlock::FileBlock(path) => path,
        }
    }
}
*/

pub struct CacheItem {
    pub meta: Metadata,
    pub data: DataBlock,
    pub mime: Mime,
    pub version: u32,
    pub expire: Option<Duration>,
    //    pub etag: ETag,
}
/*
pub fn etag_calculate(meta: &Metadata, real_len: u64) -> anyhow::Result<ETag> {
    let etag = match meta.modified().map(|modified| {
        modified
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Modified is earlier than time::UNIX_EPOCH!")
    }) {
        Ok(modified) => ETag::from_str(&format!("{:x}-{:x}", modified.as_secs(), real_len))?,
        _ => ETag::from_str(&format!("{:?}", real_len))?,
    };
    Ok(etag)
}*/

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    #[test]
    fn test_extend() {
        let mut hash = HashMap::new();
        hash.insert(1, 1);
        let mut hash2 = HashMap::new();
        hash2.insert(1, 2);
        hash.extend(hash2);
        println!("hash2 {:?}", hash.get(&1));
    }
}
