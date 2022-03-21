use crate::config::CacheConfig;
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
    conf: CacheConfig,
    data: DashMap<String, HashMap<String, Arc<CacheItem>>>,
    expire_config: HashMap<String, Duration>,
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
    pub fn new(conf: CacheConfig) -> Self {
        let expire_config: HashMap<String, Duration> = conf
            .client_cache
            .clone()
            .into_iter()
            .map(|item| {
                item.extension_names
                    .into_iter()
                    .map(|extension_name| (extension_name, item.expire.clone()))
                    .collect::<Vec<(String, Duration)>>()
            })
            .flatten()
            .collect();

        FileCache {
            conf,
            data: DashMap::new(),
            expire_config,
        }
    }
    pub fn update(
        &self,
        domain: String,
        data: HashMap<String, Arc<CacheItem>>,
    ) -> Option<HashMap<String, Arc<CacheItem>>> {
        self.data.insert(domain, data)
    }

    pub fn cache_dir(&self, path: &PathBuf) -> anyhow::Result<HashMap<String, Arc<CacheItem>>> {
        let prefix = path
            .to_str()
            .map(|x| Ok(format!("{}/", x.to_string())))
            .unwrap_or(Err(anyhow!("can not parse path")))?;
        tracing::info!("prepare to cache_dir: {}", &prefix);
        let result: HashMap<String, Arc<CacheItem>> = WalkDir::new(path)
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
                            let data_block = if self.conf.max_size.is_none()
                                || self.conf.max_size.filter(|size| *size > metadata.len()).is_some() {
                                tracing::debug!("file block:{:?}", entry_path.display());
                                DataBlock::FileBlock(ArcPath(Arc::new(entry_path)))
                            } else {
                                let (bytes, compressed) = if self.conf.compression
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
                                tracing::debug!("cache block:{:?}, compressed:{}", entry_path.display(), compressed);
                                DataBlock::CacheBlock {
                                    bytes,
                                    compressed,
                                    path: ArcPath(Arc::new(entry_path)),
                                }
                            };
                            (
                                key,
                                Arc::new(CacheItem {
                                    mime,
                                    meta: metadata,
                                    data: data_block,
                                    expire: self.expire_config.get(&extension_name).cloned(),
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

    pub fn get_item(&self, domain: &str, path: &str) -> Option<Arc<CacheItem>> {
        self.data
            .get(domain)
            .map(|x| x.get(path).map(Arc::clone))
            .flatten()
    }
}

pub enum DataBlock {
    CacheBlock {
        bytes: Bytes,
        compressed: bool,
        path: ArcPath,
    },
    // for use warp
    FileBlock(ArcPath),
}

pub struct CacheItem {
    pub meta: Metadata,
    pub data: DataBlock,
    pub mime: Mime,
    pub expire: Option<Duration>,
}
