use anyhow::anyhow;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use walkdir::WalkDir;

pub struct FileCache {
    data: DashMap<String, HashMap<String, Arc<CacheItem>>>,
}

pub struct DomainCacheConfig {}

impl Default for FileCache {
    fn default() -> Self {
        Self::new()
    }
}

impl FileCache {
    pub fn new() -> Self {
        FileCache {
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
                                && (version > k.version && version - k.version > 2
                                    || version < k.version)
                            {
                                None
                            } else {
                                Some((v.clone(), k.clone()))
                            }
                        }
                        None => {
                            if version > k.version && version - k.version > 2 || version < k.version
                            {
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
        _domain: &str, //www.example.com
        sub_path: Option<&str>,
        version: u32,
        path: &PathBuf,
    ) -> anyhow::Result<HashMap<String, Arc<CacheItem>>> {
        let _ = path
            .to_str()
            .map(|x| Ok(format!("{x}/")))
            .unwrap_or(Err(anyhow!("can not parse path")))?;
        let parent = path.clone();
        let mut result: HashMap<String, Arc<CacheItem>> = WalkDir::new(path)
            .min_depth(1)
            .into_iter()
            .filter_map(|x| x.ok())
            .filter_map(|entry| {
                if let Ok(metadata) = entry.metadata()
                    && metadata.is_file()
                    && let Ok(key) = entry.path().strip_prefix(&parent)
                {
                    let key = key
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy())
                        .collect::<Vec<_>>()
                        .join("/");
                    let key = sub_path
                        .map(|sub_path| format!("{sub_path}/{key}"))
                        .unwrap_or(key);
                    return Some((
                        key,
                        Arc::new(CacheItem {
                            data: entry.path().to_path_buf(),
                            version,
                        }),
                    ));
                }
                None
            })
            .collect();

        match sub_path {
            Some(key_prefix) => {
                let index_opt = result
                    .get(&format!("{key_prefix}/index.html"))
                    .or_else(|| result.get(&format!("{key_prefix}/index.htm")))
                    .cloned();
                if let Some(v) = index_opt {
                    result.insert(format!("{key_prefix}/"), v.clone());
                    // result.insert(key_prefix.to_string(), v); //GitHub Action CI would trigger This, but I could not trigger this in my compute
                }
            }
            None => {
                let index_opt = result
                    .get("index.html")
                    .or_else(|| result.get("index.htm"))
                    .cloned();
                if let Some(v) = index_opt {
                    result.insert("".to_string(), v.clone());
                    //result.insert("/".to_string(), v);
                }
            }
        }

        Ok(result)
    }

    pub fn get_item(&self, host: &str, path: &str) -> Option<Arc<CacheItem>> {
        self.data.get(host).and_then(|x| x.get(path).cloned())
    }
    pub fn get_all_keys(&self, host: &str) -> Vec<String> {
        self.data
            .get(host)
            .map(|x| {
                let keys = x.value().keys();
                keys.map(|x| x.to_string()).collect()
            })
            .unwrap_or_default()
    }
    pub fn delete_by_host(&self, host: &str, sub_dir: Option<String>, version: Option<u32>) {
        match (sub_dir, version) {
            (None, None) => {
                self.data.remove(host);
            }
            (sub_dir, version) => {
                let map = self.data.get(host).map(|x| {
                    x.iter()
                        .filter_map(|(key, value)| {
                            if sub_dir
                                .as_ref()
                                .map(|sub_dir| key.starts_with(sub_dir))
                                .unwrap_or(true)
                                && version
                                    .as_ref()
                                    .map(|version| value.version == *version)
                                    .unwrap_or(true)
                            {
                                None
                            } else {
                                Some((key.clone(), value.clone()))
                            }
                        })
                        .collect::<HashMap<String, Arc<CacheItem>>>()
                });
                if let Some(keys) = map {
                    self.data.insert(host.to_string(), keys);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArcPath(pub Arc<PathBuf>);

pub struct CacheItem {
    pub data: PathBuf,
    pub version: u32,
}

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
