use anyhow::anyhow;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) const URI_REGEX_STR: &str =
    "[a-zA-Z0-9][-a-zA-Z0-9]{0,62}(\\.[a-zA-Z0-9][-a-zA-Z0-9]{0,62})+\\.?";
//"[a-zA-Z0-9][-a-zA-Z0-9]{0,62}(\\.[a-zA-Z0-9][-a-zA-Z0-9]{0,62})+$";

pub struct DomainStorage {
    meta: HashMap<String, PathBuf>,
    prefix: PathBuf,
}

impl DomainStorage {
    pub fn init<T: AsRef<Path>>(path_prefix: T) -> anyhow::Result<DomainStorage> {
        lazy_static! {
            static ref URI_REGEX: Regex = Regex::new(URI_REGEX_STR).unwrap();
        }
        let path_prefix = path_prefix.as_ref();
        let path_prefix_buf = path_prefix.to_path_buf();
        if path_prefix.exists() {
            let mut domain_version: HashMap<String, PathBuf> = HashMap::new();
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
                        info!("serve domain: {},version: {}", domain_dir_name, max_version);
                        domain_version.insert(
                            domain_dir_name.to_owned(),
                            path_prefix_buf
                                .join(domain_dir_name)
                                .join(max_version.to_string()),
                        );
                    }
                }
            }

            Ok(DomainStorage {
                meta: domain_version,
                prefix: path_prefix.to_path_buf(),
            })
        } else {
            Err(anyhow!("{:?} does not exist", path_prefix))
        }
    }

    pub fn get_version_path(&self, host: &str) -> Option<&PathBuf> {
        self.meta.get(host)
    }
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
