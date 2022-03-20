use crate::config::CacheConfig;
use anyhow::anyhow;
use dashmap::DashMap;
use flate2::read::GzEncoder;
use flate2::Compression;
use hyper::body::Bytes;
use if_chain::if_chain;
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
            .unwrap_or(Vec::new())
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
        //TODO: what about error handle? need refactor
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
                            let data_block = if_chain!(
                                if let Some(max_size) = self.conf.max_size;
                                if max_size < metadata.len();
                                then {
                                    DataBlock::FileBlock(ArcPath(Arc::new(entry_path)))
                                } else {
                                    if self.conf.compression.unwrap_or(false) && COMPRESSION_FILE_TYPE.contains(&extension_name){
                                        let mut encoded_bytes = Vec::new();
                                        let mut encoder = GzEncoder::new(&bytes[..], Compression::default());
                                        encoder.read_to_end(&mut encoded_bytes).unwrap();
                                        DataBlock::CacheBlock{bytes:Bytes::from(encoded_bytes),compressed: true}
                                    } else {
                                        DataBlock::CacheBlock{bytes:Bytes::from(bytes), compressed:false}
                                    }
                                }
                            );
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
    CacheBlock { bytes: Bytes, compressed: bool },
    // for use warp
    FileBlock(ArcPath),
}

pub struct CacheItem {
    pub meta: Metadata,
    pub data: DataBlock,
    pub mime: Mime,
    pub expire: Option<Duration>,
}
