use crate::config::CacheConfig;
use anyhow::anyhow;
use dashmap::DashMap;
use hyper::body::Bytes;
use mime::Mime;
use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::Arc;
use walkdir::WalkDir;
use warp::fs::ArcPath;

pub struct FileCache {
    conf: CacheConfig,
    data: DashMap<String, HashMap<String, Arc<CacheItem>>>,
}

impl FileCache {
    pub fn new(conf: CacheConfig) -> Self {
        FileCache {
            conf,
            data: DashMap::new(),
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
                            if let Some(max_size) = self.conf.max_size {
                                if max_size < metadata.len() {
                                    return (
                                        key,
                                        Arc::new(CacheItem {
                                            mime,
                                            meta: metadata,
                                            data: DataBlock::FileBlock(ArcPath(Arc::new(
                                                entry_path,
                                            ))),
                                        }),
                                    );
                                }
                            }
                            (
                                key,
                                Arc::new(CacheItem {
                                    mime,
                                    meta: metadata,
                                    data: DataBlock::CacheBlock(Bytes::from(bytes)),
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
    CacheBlock(Bytes),
    // for use warp
    FileBlock(ArcPath),
}

pub struct CacheItem {
    pub meta: Metadata,
    pub data: DataBlock,
    pub mime: Mime,
}
