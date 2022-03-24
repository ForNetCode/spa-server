use crate::file_cache::{CacheItem, FileCache};
use anyhow::anyhow;
use dashmap::DashMap;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) const URI_REGEX_STR: &str =
    "[a-zA-Z0-9][-a-zA-Z0-9]{0,62}(\\.[a-zA-Z0-9][-a-zA-Z0-9]{0,62})+\\.?";
//"[a-zA-Z0-9][-a-zA-Z0-9]{0,62}(\\.[a-zA-Z0-9][-a-zA-Z0-9]{0,62})+$";

lazy_static! {
    pub static ref URI_REGEX: Regex = Regex::new(URI_REGEX_STR).unwrap();
}

pub struct DomainStorage {
    meta: DashMap<String, (PathBuf, i32)>,
    prefix: PathBuf,
    cache: FileCache,
}

impl DomainStorage {
    pub fn init<T: AsRef<Path>>(path_prefix: T, cache: FileCache) -> anyhow::Result<DomainStorage> {
        let path_prefix = path_prefix.as_ref();
        let path_prefix_buf = path_prefix.to_path_buf();
        if path_prefix.exists() {
            let domain_version: DashMap<String, (PathBuf, i32)> = DashMap::new();
            let domain_dirs = fs::read_dir(path_prefix)?;
            for domain_dir in domain_dirs {
                let domain_dir = domain_dir?;
                let metadata = domain_dir.metadata()?;
                let domain_dir_name = domain_dir.file_name();
                let domain_dir_name = domain_dir_name.to_str().unwrap();

                if metadata.is_dir() && URI_REGEX.is_match(domain_dir_name) {
                    let mut max_version = 0;
                    for version_dir_entry in fs::read_dir(domain_dir.path())? {
                        let version_dir_entry = version_dir_entry?;

                        if let Some(version_dir) = version_dir_entry
                            .file_name()
                            .to_str()
                            .map(|file_name| file_name.parse::<i32>().ok())
                            .flatten()
                        {
                            if max_version < version_dir {
                                max_version = version_dir
                            }
                        }
                    }
                    if max_version > 0 {
                        tracing::info!(
                            "serve domain: {},version: {}",
                            domain_dir_name,
                            max_version
                        );
                        let path_buf = path_prefix_buf
                            .join(domain_dir_name)
                            .join(max_version.to_string());
                        let data = cache.cache_dir(&path_buf)?;
                        cache.update(domain_dir_name.to_string(), data);
                        domain_version.insert(domain_dir_name.to_owned(), (path_buf, max_version));
                    }
                }
            }
            Ok(DomainStorage {
                meta: domain_version,
                prefix: path_prefix.to_path_buf(),
                cache,
            })
        } else {
            Err(anyhow!("{:?} does not exist", path_prefix))
        }
    }

    pub fn get_file(&self, host: &str, key: &str) -> Option<Arc<CacheItem>> {
        self.cache.get_item(host, key)
    }

    pub async fn upload_domain_with_version(
        &self,
        domain: String,
        version: i32,
    ) -> anyhow::Result<()> {
        let new_path = self.prefix.join(&domain).join(version.to_string());
        if new_path.is_dir() {
            self.meta
                .insert(domain.clone(), (new_path.clone(), version));
            let data = self.cache.cache_dir(&new_path)?;
            tracing::info!("update domain:{}, version:{} ", &domain, version);
            self.cache.update(domain, data);
            Ok(())
        } else {
            Err(anyhow!("{:?} does not exits", new_path))
        }
    }

    pub fn get_version_path(&self, host: &str) -> Option<PathBuf> {
        self.meta.get(host).map(|d| d.value().0.clone())
    }

    pub fn get_new_upload_path(&self, domain: &str) -> PathBuf {
        match self.get_domain_info_by_domain(domain) {
            Some(domain_info) => {
                let max_version = domain_info.versions.iter().max().unwrap_or(&0);
                self.prefix.join(domain).join((max_version + 1).to_string())
            }
            None => self.prefix.join(domain).join(1.to_string()),
        }
    }

    pub fn get_domain_info_by_domain(&self, domain: &str) -> Option<DomainInfo> {
        self.get_domain_info()
            .into_iter()
            .find(|x| x.domain == domain)
    }

    pub fn get_domain_info(&self) -> Vec<DomainInfo> {
        let mut result: Vec<DomainInfo> = Vec::new();
        for item in self.meta.iter() {
            let (path, version) = item.value();
            let mut versions: Vec<i32> = Vec::new();
            if let Ok(version_dir) = fs::read_dir(path.parent().unwrap()) {
                for version in version_dir {
                    if let Ok(version) = version {
                        if let Some(Ok(version)) =
                            version.file_name().to_str().map(|x| x.parse::<i32>())
                        {
                            versions.push(version)
                        }
                    }
                }
            }
            result.push(DomainInfo {
                domain: item.key().to_string(),
                current_version: *version,
                versions,
            })
        }
        result
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DomainInfo {
    pub domain: String,
    pub current_version: i32,
    pub versions: Vec<i32>,
}
#[cfg(test)]
mod test {
    use crate::domain_storage::URI_REGEX_STR;
    use regex::Regex;

    #[test]
    fn test_uri_regex() {
        let uri_regex = Regex::new(URI_REGEX_STR).unwrap();
        // println!("{}", uri_regex.is_match("www.baidu.com"));
        assert!(
            uri_regex.is_match("www.baidu.com"),
            "uri_regex can not parse www.baidu.com"
        );
        assert!(!uri_regex.is_match("baidu"));
        //println!("{}", uri_regex.is_match("abc"));
        //assert!(!uri_regex.is_match("abc"));
        let r2 = Regex::new("\\.").unwrap();
        assert!(r2.is_match("."));
        assert!(!r2.is_match("x"));
    }
}
