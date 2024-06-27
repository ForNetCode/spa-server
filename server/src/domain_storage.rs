use crate::acme::ACMEManager;
use crate::config::get_host_path_from_domain;
use crate::file_cache::{CacheItem, FileCache};
use anyhow::{anyhow, bail, Context};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use lazy_static::lazy_static;
use md5::{Digest, Md5};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info};
use walkdir::{DirEntry, WalkDir};
use warp::fs::sanitize_path;

pub(crate) const URI_REGEX_STR: &str =
    "[a-zA-Z0-9][-a-zA-Z0-9]{0,62}(\\.[a-zA-Z0-9][-a-zA-Z0-9]{0,62})+\\.?";
//"[a-zA-Z0-9][-a-zA-Z0-9]{0,62}(\\.[a-zA-Z0-9][-a-zA-Z0-9]{0,62})+$";

lazy_static! {
    pub static ref URI_REGEX: Regex = Regex::new(URI_REGEX_STR).unwrap();
}

pub(crate) const UPLOADING_FILE_NAME: &str = ".SPA-Processing";
pub(crate) const MULTIPLE_WEB_FILE_NAME: &str = ".SPA-Multiple";
// pub(crate) const SINGLE_WEB_FILE_NAME: &str = ".SPA-Single";

#[derive(Debug)]
pub enum DomainMeta {
    OneWeb(PathBuf, u32),                         // path, u32
    MultipleWeb(DashMap<String, (PathBuf, u32)>), // path, u32  {a/b: ($path_prefix/$domain/a/b/${serving_version},  serving_version)}
}

// TODO: add write locker for domain storage or domain. to keep it free from multiple update at same time.
pub struct DomainStorage {
    meta: DashMap<String, DomainMeta>,
    prefix: PathBuf,
    cache: FileCache, // {[${domain}/${multiple_path}|$domain]: ${absolute_path}/version}
    uploading_status: DashMap<String, u32>,
}

impl DomainStorage {
    pub fn init<T: AsRef<Path>>(path_prefix: T, cache: FileCache) -> anyhow::Result<DomainStorage> {
        let path_prefix = path_prefix.as_ref();
        let path_prefix_buf = path_prefix.to_path_buf();
        if path_prefix.exists() {
            let domain_version: DashMap<String, DomainMeta> = DashMap::new();
            let uploading_status: DashMap<String, u32> = DashMap::new();

            let domain_dirs = fs::read_dir(path_prefix)?;
            for domain_dir in domain_dirs {
                let domain_dir = domain_dir?;
                let metadata = domain_dir.metadata()?;
                let domain_dir_name = domain_dir.file_name();
                //domain_dir_name = www.example.com
                let domain_dir_name = domain_dir_name.to_str().unwrap();
                if metadata.is_dir() && URI_REGEX.is_match(domain_dir_name) {
                    let domain_dir = domain_dir.path();
                    // domain_dir = ${path_prefix}/www.example.com
                    let multiple_web_file = domain_dir.join(MULTIPLE_WEB_FILE_NAME);
                    // confirm it's multiple web
                    if multiple_web_file.exists() {
                        let sub_dirs = Self::get_multiple_path_data(
                            &domain_dir,
                            domain_dir_name,
                            &multiple_web_file,
                        )?;
                        // sub_dirs = [("${path_prefix}/www.example.com/a/b", "www.example.com/a/b", a/b)]
                        for (sub_dir, domain_with_sub_path, sub_path) in sub_dirs {
                            let (uploading_version, max_version) =
                                Self::get_meta_info(&sub_dir, &domain_with_sub_path)?;
                            if let Some(uploading_version) = uploading_version {
                                uploading_status
                                    .insert(domain_with_sub_path.clone(), uploading_version);
                            }

                            if let Some(version) = max_version {
                                for version in Self::get_init_version(version) {
                                    let path_buf = sub_dir.join(version.to_string());
                                    if sub_dir
                                        .join(version.to_string())
                                        .join(UPLOADING_FILE_NAME)
                                        .exists()
                                    {
                                        continue;
                                    }
                                    let data = cache.cache_dir(
                                        domain_dir_name,
                                        Some(sub_path.as_str()),
                                        version,
                                        &path_buf,
                                    )?;
                                    cache.update(
                                        domain_dir_name.to_string(),
                                        Some(sub_path.as_str()),
                                        version,
                                        data,
                                    );
                                }
                                let path_buf = sub_dir.join(version.to_string());
                                match domain_version.get_mut(domain_dir_name) {
                                    Some(mut domain_meta) => {
                                        match domain_meta.value_mut() {
                                            DomainMeta::MultipleWeb(ref mut map) => {
                                                map.insert(sub_path.clone(), (path_buf, version));
                                            }
                                            DomainMeta::OneWeb(..) => {
                                                panic!("init failure, {sub_dir:?} should be multiple web");
                                            }
                                        }
                                    }
                                    None => {
                                        let map = DashMap::new();
                                        map.insert(sub_path.clone(), (path_buf, version));
                                        domain_version.insert(
                                            domain_dir_name.to_string(),
                                            DomainMeta::MultipleWeb(map),
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        let (uploading_version, max_version) =
                            Self::get_meta_info(&domain_dir, domain_dir_name)?;
                        if let Some(uploading_version) = uploading_version {
                            uploading_status.insert(domain_dir_name.to_string(), uploading_version);
                        }
                        if let Some(version) = max_version {
                            for version in Self::get_init_version(version) {
                                let path_buf = path_prefix_buf
                                    .join(domain_dir_name)
                                    .join(version.to_string());
                                let data =
                                    cache.cache_dir(domain_dir_name, None, version, &path_buf)?;
                                cache.update(domain_dir_name.to_string(), None, version, data);
                            }
                            let path_buf = path_prefix_buf
                                .join(domain_dir_name)
                                .join(version.to_string());
                            domain_version.insert(
                                domain_dir_name.to_string(),
                                DomainMeta::OneWeb(path_buf, version),
                            );
                        }
                    }
                }
            }
            Ok(DomainStorage {
                meta: domain_version,
                prefix: path_prefix.to_path_buf(),
                cache,
                uploading_status,
            })
        } else {
            Err(anyhow!("{:?} does not exist", path_prefix))
        }
    }

    fn get_init_version(version: u32) -> RangeInclusive<u32> {
        if version > 2 {
            version - 2..=version
        } else {
            1..=version
        }
    }

    fn get_multiple_path_data<'a>(
        domain_dir: &PathBuf,
        domain_dir_name: &'a str,
        multiple_web_path: &PathBuf,
    ) -> anyhow::Result<Vec<(PathBuf, String, String)>> {
        let sub_dirs = fs::read_to_string(multiple_web_path)
            .with_context(|| format!("read path fail: {:?}", &multiple_web_path))?;
        let sub_dirs = sub_dirs
            .lines()
            .map(|path| {
                (
                    domain_dir.join(path),
                    format!("{domain_dir_name}/{path}"),
                    path.to_string(),
                )
            })
            .collect();
        Ok(sub_dirs)
    }

    fn get_meta_info(
        domain_dir: &PathBuf,
        domain_dir_name: &str,
    ) -> anyhow::Result<(Option<u32>, Option<u32>)> {
        let mut max_version = 0;
        let mut uploading_version = None;
        if !domain_dir.exists() {
            return Ok((None, None));
        }
        for version_dir_entry in fs::read_dir(domain_dir)? {
            let version_dir_entry = version_dir_entry?;
            if let Some(version_dir) = version_dir_entry
                .file_name()
                .to_str()
                .and_then(|file_name| file_name.parse::<u32>().ok())
            {
                let mut path = version_dir_entry.path();
                path.push(UPLOADING_FILE_NAME);
                // this directory is in uploading
                if path.exists() {
                    //
                    uploading_version = Some(version_dir);
                    //uploading_status.insert(domain_dir_name.to_string(), version_dir);
                } else if max_version < version_dir {
                    max_version = version_dir;
                }
            }
        }
        if max_version > 0 {
            info!("serve: {},version: {}", domain_dir_name, max_version);
            return Ok((uploading_version, Some(max_version)));
        }
        Ok((uploading_version, None))
    }
    pub fn get_file(&self, host: &str, key: &str) -> Option<Arc<CacheItem>> {
        self.cache.get_item(host, key)
    }

    pub async fn upload_domain_with_version(
        &self,
        domain: String,
        version: Option<u32>,
    ) -> anyhow::Result<u32> {
        //TODO: check if multiple
        let version = if let Some(version) = version {
            version
        } else {
            let max_version_opt = self
                .get_domain_info_by_domain(&domain)
                .map(|x| x.versions)
                .unwrap_or_default()
                .into_iter()
                .max();
            if let Some(max_version) = max_version_opt {
                max_version
            } else {
                return Err(anyhow!(
                    "domain:{} does not exist version, please check if domain err",
                    &domain
                ));
            }
        };
        let new_path = self.prefix.join(&domain).join(version.to_string());
        if self
            .uploading_status
            .get(&domain)
            .filter(|x| *x.value() == version)
            .is_some()
        {
            Err(anyhow!(
                "domain:{},version:{} is uploading now, please finish it firstly",
                domain,
                version
            ))
        } else if new_path.is_dir() {
            info!(
                "begin to update domain:{}, version:{}, putting files to cache",
                &domain, version
            );
            let (host, path) = get_host_path_from_domain(&domain);
            match self.meta.get(host) {
                Some(domain_meta) => {
                    //TODO: check path and DomainMeta if is pattern
                    match domain_meta.value() {
                        DomainMeta::OneWeb(..) => {
                            //must keep this drop, otherwise, deadlock.
                            drop(domain_meta);
                            self.meta.insert(
                                host.to_string(),
                                DomainMeta::OneWeb(new_path.clone(), version),
                            );
                        }
                        DomainMeta::MultipleWeb(map) => {
                            let multiple_file = self.prefix.join(host).join(MULTIPLE_WEB_FILE_NAME);
                            if multiple_file.exists() {
                                let mut file = OpenOptions::new()
                                    .append(true)
                                    .read(true)
                                    .open(multiple_file)?;
                                let mut multiple_path = String::new();
                                file.read_to_string(&mut multiple_path)?;
                                if !multiple_path.lines().any(|x| x == path) {
                                    writeln!(file, "{}", path)?;
                                }
                            }
                            map.insert(path.to_string(), (new_path.clone(), version));
                        }
                    };
                }
                None => {
                    //TODO: check if MULTIPLE_WEB_FILE_NAME exists
                    if path.is_empty() {
                        self.meta.insert(
                            host.to_string(),
                            DomainMeta::OneWeb(new_path.clone(), version),
                        );
                    } else {
                        let multiple_file = self.prefix.join(&host).join(MULTIPLE_WEB_FILE_NAME);
                        let mut file = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .read(true)
                            .open(&multiple_file)?;

                        let mut multiple_path = String::new();
                        file.read_to_string(&mut multiple_path)?;
                        if !multiple_path.lines().any(|x| x == path) {
                            writeln!(file, "{}", path)?;
                        }
                        let map = DashMap::new();

                        info!("create multiple_file {multiple_file:?}, append {path}");
                        map.insert(path.to_string(), (new_path.clone(), version));

                        self.meta
                            .insert(host.to_string(), DomainMeta::MultipleWeb(map));
                    }
                }
            };
            let path = if path.is_empty() { None } else { Some(path) };
            let data = self.cache.cache_dir(host, path, version, &new_path)?;
            self.cache.update(host.to_string(), path, version, data);
            debug!(
                "domain: {host} all keys:{:?}",
                self.cache.get_all_keys(host)
            );
            info!(
                "update domain:{}, sub_path: {:?} ,version:{} finish!",
                host, path, version
            );
            Ok(version)
        } else {
            Err(anyhow!("{:?} does not exits", new_path))
        }
    }

    pub fn get_version_path(&self, host: &str, version: u32) -> PathBuf {
        let mut prefix = self.prefix.clone();
        prefix.push(host);
        prefix.push(version.to_string());
        prefix
    }

    pub fn get_upload_position(&self, domain: &str) -> anyhow::Result<UploadDomainPosition> {
        self.check_if_can_upload(domain)?;
        let result = if let Some(version) = self.uploading_status.get(domain).map(|x| *x.value()) {
            UploadDomainPosition {
                path: self.get_version_path(domain, version),
                version,
                status: GetDomainPositionStatus::InUploading,
            }
        } else {
            match self.get_domain_info_by_domain(domain) {
                Some(domain_info) => {
                    let max_version = domain_info.versions.iter().max().unwrap_or(&0);
                    let version = max_version + 1u32;
                    UploadDomainPosition {
                        path: self.prefix.join(domain).join(version.to_string()),
                        version,
                        status: GetDomainPositionStatus::NewVersion,
                    }
                }
                None => {
                    let version = 1;
                    UploadDomainPosition {
                        version,
                        path: self.prefix.join(domain).join(version.to_string()),
                        status: GetDomainPositionStatus::NewDomain,
                    }
                }
            }
        };
        Ok(result)
    }

    pub fn get_domain_serving_version(&self, domain: &str) -> Option<u32> {
        let (host, path) = get_host_path_from_domain(&domain);
        let domain_meta = self.meta.get(host)?;
        match domain_meta.value() {
            DomainMeta::OneWeb(_, version) if path == "" => Some(version.clone()),
            DomainMeta::MultipleWeb(map) if path != "" => map.get(path).map(|v| {
                let (_, v) = v.value();
                v.clone()
            }),
            _ => None,
        }
    }

    //domain: www.example.com|www.example.com/a/b
    pub fn get_domain_info_by_domain(&self, domain: &str) -> Option<DomainInfo> {
        let path = self.prefix.join(domain);
        let versions: Vec<u32> = WalkDir::new(&path)
            .max_depth(1)
            .min_depth(1)
            .into_iter()
            .filter_map(|version_entity| {
                let version_entity = version_entity.ok()?;
                let version = version_entity.file_name().to_str()?.parse::<u32>();
                version.ok()
            })
            .collect();
        if versions.is_empty() {
            None
        } else {
            let domain = domain.to_string();
            let current_version = self.get_domain_serving_version(&domain);
            /*
            let web_path: Vec<String> = if let Some(current_version) = &current_version {
                let path = path.join(current_version.to_string());
                let path_str = path.display().to_string();
                WalkDir::new(&path)
                    .into_iter()
                    .filter_map(|dir_entry| {
                        let path = dir_entry.ok()?;
                        let path = path.path();
                        if path.is_file() {
                            let path = format!(
                                "{domain}/{}",
                                path.display().to_string().replace(&path_str, "")
                            );
                            return Some(path);
                        }
                        None
                    })
                    .collect()
            } else {
                Vec::new()
            };*/
            Some(DomainInfo {
                domain,
                current_version,
                versions,
                // web_path,
            })
        }
    }

    pub fn get_domain_info(&self) -> anyhow::Result<Vec<DomainInfo>> {
        let ret: Vec<DomainInfo> = WalkDir::new(&self.prefix)
            .max_depth(1)
            .min_depth(1)
            .into_iter()
            .filter_map(|dir_entity| {
                let dir_entity = dir_entity.ok()?;
                let domain_dir_name = dir_entity.file_name().to_str()?;
                if dir_entity.metadata().ok()?.is_dir() && URI_REGEX.is_match(domain_dir_name) {
                    let domain_dir = dir_entity.path().to_path_buf();
                    let multiple_web_path = domain_dir.join(MULTIPLE_WEB_FILE_NAME);
                    if multiple_web_path.is_file() {
                        //[("${path_prefix}/www.example.com/a/b", "www.example.com/a/b", a/b)]
                        let sub_dirs = Self::get_multiple_path_data(
                            &domain_dir,
                            domain_dir_name,
                            &multiple_web_path,
                        )
                        .ok()?;
                        let result: Vec<DomainInfo> = sub_dirs
                            .iter()
                            .filter_map(|(_, domain_with_sub_path, _)| {
                                self.get_domain_info_by_domain(&domain_with_sub_path)
                            })
                            .collect();
                        Some(result)
                    } else {
                        self.get_domain_info_by_domain(domain_dir_name)
                            .map(|v| vec![v])
                    }
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        Ok(ret)
    }
    fn check_is_in_upload_process(&self, domain: &str, version: &u32) -> bool {
        self.uploading_status
            .get(domain)
            .filter(|x| x.value() == version)
            .is_some()
    }

    pub fn get_files_metadata(
        &self,
        domain: String,
        version: u32,
    ) -> anyhow::Result<Vec<ShortMetaData>> {
        let path_buf = self.get_version_path(&domain, version);
        if path_buf.exists() {
            let prefix = path_buf
                .to_str()
                .map(|x| Ok(format!("{}/", x.to_string())))
                .unwrap_or(Err(anyhow!("can not parse path")))?;
            let mut byte_buffer = vec![0u8; 1024 * 1024];

            fn get_short_metadata(
                entry: DirEntry,
                prefix: &str,
                byte_buffer: &mut Vec<u8>,
            ) -> Option<ShortMetaData> {
                let x = entry.path().to_str()?;
                let key = x.replace(prefix, "");
                if let Ok(meta) = entry.metadata() {
                    let md5 = md5_file(entry.path(), byte_buffer)?;
                    let ret = ShortMetaData {
                        path: key,
                        md5,
                        length: meta.len(),
                    };
                    tracing::trace!("ShortMetaData {:?}", ret);
                    Some(ret)
                } else {
                    None
                }
            }
            let ret = WalkDir::new(&path_buf)
                .into_iter()
                .filter_map(|x| x.ok())
                .filter(|x| x.file_name() != UPLOADING_FILE_NAME && x.file_type().is_file())
                .filter_map(|entry| get_short_metadata(entry, &prefix, &mut byte_buffer))
                .collect::<Vec<ShortMetaData>>();
            Ok(ret)
        } else {
            Ok(Vec::new())
        }
    }
    pub fn check_if_empty_index(&self, host: &str, path: &str) -> bool {
        match self.meta.get(host) {
            Some(v) => match v.value() {
                DomainMeta::OneWeb { .. } => path.is_empty(),
                DomainMeta::MultipleWeb(map) => {
                    if path.len() > 1 {
                        let path = &path[1..];
                        map.contains_key(path)
                    } else {
                        map.contains_key(path)
                    }
                }
            },
            None => {
                debug!("{host} {path} does not exists in meta");
                false
            }
        }
    }

    pub fn save_file(
        &self,
        domain: String,
        version: u32,
        path: String,
        data: Vec<u8>,
    ) -> anyhow::Result<()> {
        if self.check_is_in_upload_process(&domain, &version) {
            let file_path = sanitize_path(self.get_version_path(&domain, version), &path)
                .map_err(|_| anyhow!("path error"))?;
            let parent_path = file_path
                .parent()
                .ok_or_else(|| anyhow!("parent path of:{:?} does not exists", &file_path))?;
            if !parent_path.exists() {
                fs::create_dir_all(parent_path)?;
            };
            let mut file = File::create(file_path)?;
            file.write_all(&data)?;
            Ok(())
        } else {
            Err(anyhow!(
                "domain:{}, version:{} can't be uploaded file, it's not in the status allowing to upload file",
                domain,
                version,
            ))
        }
    }

    pub async fn update_uploading_status(
        &self,
        domain: String,
        version: u32,
        uploading_status: UploadingStatus,
        acme_manager: &ACMEManager,
    ) -> anyhow::Result<()> {
        // TODO: check if multiple and domain match
        if let Some(uploading_version) = self.uploading_status.get(&domain).map(|v| *v.value()) {
            if uploading_version != version {
                return Err(anyhow!(
                    "domain:{}, version:{} is in uploading, please finish it firstly",
                    domain,
                    uploading_version,
                ));
            }
            if uploading_status == UploadingStatus::Finish {
                let mut p = self.get_version_path(&domain, version);
                p.push(UPLOADING_FILE_NAME);
                fs::remove_file(p)?;
                self.uploading_status
                    .remove_if(&domain, |_, v| *v == version);
                info!(
                    "domain:{}, version:{} change to upload status:finish",
                    domain, version
                );
                acme_manager
                    .add_new_domain(get_host_path_from_domain(&domain).0)
                    .await;
            }
        } else if uploading_status == UploadingStatus::Uploading {
            if self
                .get_domain_serving_version(&domain)
                .filter(|x| *x == version)
                .is_some()
            {
                return Err(anyhow!(
                    "domain:{}, version:{} is in serving, can not change upload status",
                    domain,
                    version,
                ));
            }
            let mut p = self.get_version_path(&domain, version);
            let (host, path) = get_host_path_from_domain(&domain);
            let multiple = self.prefix.join(host).join(MULTIPLE_WEB_FILE_NAME);
            if !path.is_empty() {
                if !multiple.exists() {
                    let parent = self.prefix.join(host);
                    if !parent.exists() && fs::create_dir_all(&parent).is_err() {
                        bail!(
                            "create host directory {} failure",
                            parent.display().to_string()
                        )
                    }
                    if File::create_new(multiple).is_err() {
                        bail!("files in same domain should not create at same time")
                    }
                }
            } else if multiple.exists() {
                bail!("This already has multiple SPA, should not upload single SPA at top path")
            }
            fs::create_dir_all(&p)?;
            p.push(UPLOADING_FILE_NAME);
            File::create(p)?;
            info!(
                "domain:{}, version:{} change to upload status:uploading",
                domain, version
            );
            self.uploading_status.insert(domain, version);
        } else {
            let mut p = self.get_version_path(&domain, version);
            p.push(UPLOADING_FILE_NAME);
            fs::remove_file(p)?;
            self.uploading_status
                .remove_if(&domain, |_, v| *v == version);
            info!(
                "domain:{}, version:{} change to upload status:finish",
                domain, version
            );
        }
        Ok(())
    }

    // No Check, who use this must check if illegal to delete files
    pub fn remove_domain_version(
        &self,
        domain: &str,
        version: Option<u32>,
    ) -> anyhow::Result<bool> {
        let mut path = self.prefix.join(domain);
        if let Some(version) = version {
            path = path.join(version.to_string());
            if path.exists() {
                fs::remove_dir_all(path)?;
                return Ok(true);
            }
        } else if path.exists() {
            fs::remove_dir_all(path)?;
            return Ok(true);
        }
        let (host, path) = match domain.split_once('/') {
            Some((host, path)) => (host, Some(path.to_string())),
            None => (domain, None),
        };
        self.cache.delete_by_host(host, path, version);
        Ok(false)
    }

    pub fn check_if_can_upload(&self, domain:&str) -> anyhow::Result<()> {
        match get_host_path_from_domain(domain)  {
            (host, "") => {// single
                if self.meta.get(host).is_some_and(|x| matches!(x.value(), DomainMeta::MultipleWeb(..))) {
                    bail!("this domain already has multiple SPA!")
                }
            },
            (host, _) => {
                if self.meta.get(host).is_some_and(|x| matches!(x.value(), DomainMeta::OneWeb(..))) {
                    bail!("this domain already has single SPA!")
                }
            }
        };
        Ok(())
    }
}

pub fn md5_file(path: impl AsRef<Path>, byte_buffer: &mut Vec<u8>) -> Option<String> {
    File::open(path)
        .ok()
        .map(|mut f| {
            let mut hasher = Md5::new();
            //if file_size > 1024 * 1024 {
            //1Mb
            loop {
                let n = f.read(byte_buffer).ok()?;
                let valid_buf_slice = &byte_buffer[..n];
                if n == 0 {
                    break;
                }
                hasher.update(valid_buf_slice);
            }
            Some(format!("{:x}", hasher.finalize()))
        })
        .flatten()
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DomainInfo {
    pub domain: String, // www.example.com|www.example.com/a/b
    pub current_version: Option<u32>,
    pub versions: Vec<u32>,
    //pub uploading_version: Vec<u32>, //TODO: add uploading_versions
    //pub web_path: Vec<String>, // [www.example.com/index.html|www.example.com/a/b/index.html,...]
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ShortMetaData {
    pub path: String,
    pub md5: String,
    pub length: u64,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum UploadingStatus {
    Uploading = 0,
    Finish = 1,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum GetDomainPositionStatus {
    NewDomain = 0,
    NewVersion = 1,
    InUploading = 2,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UploadDomainPosition {
    pub path: PathBuf,
    pub version: u32,
    pub status: GetDomainPositionStatus,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CertInfo {
    pub begin: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub host: String,
}

#[cfg(test)]
mod test {
    use crate::config::Config;
    use crate::domain_storage::{DomainStorage, URI_REGEX_STR};
    use crate::file_cache::FileCache;
    use hyper::Uri;
    use regex::Regex;
    use std::env;
    use std::fs::OpenOptions;
    use std::io::Read;
    use std::ops::RangeInclusive;
    use std::path::PathBuf;
    use std::str::FromStr;

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
    #[test]
    fn test_path() {
        assert_eq!(
            PathBuf::from("/etc").display().to_string(),
            "/etc".to_string()
        );
        assert_eq!(
            PathBuf::from("/etc/").display().to_string(),
            "/etc/".to_string()
        );
    }

    #[test]
    fn test_parse() {
        let z = Uri::from_str("http://www.example.com/a/b").unwrap();
        println!("{z:?}, {:?}, {:?}", z.path(), z.host());

        let z = Uri::from_str("http://www.example.com").unwrap();
        println!("{z:?}, {:?}, {:?}", z.path(), z.host());
        let z = "www.example.com/abc/cde".split_once('/');
        println!("{z:?}");
        let z = "www.example.com/".split_once('/');
        println!("{z:?}");
    }
    #[test]
    fn test_lines() {
        let z = "a\nb\n".to_string();
        let z = z.lines();
        let z: Vec<&str> = z.collect();
        println!("{}", z.len());
    }

    #[test]
    fn get_init_version_test() {
        assert_eq!(DomainStorage::get_init_version(3), 1..=3);
        assert_eq!(DomainStorage::get_init_version(2), 1..=2);
        assert_eq!(DomainStorage::get_init_version(4), 2..=4);
        assert!(RangeInclusive::new(1, 0).is_empty());
    }
    // #[test]
    // fn test_path() {
    //     let path = PathBuf::from("/");
    //     assert!(path.join("usr/lib/pam/").is_dir());
    //     println!("{:?}", path.join("usr/lib/pam/").to_str());
    // }

    #[ignore]
    #[test]
    fn test_domain_storage_get_domain_info() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../test/config.test.conf");
        env::set_var("SPA_CONFIG", path.display().to_string());
        let mut config = Config::load().unwrap();
        config.file_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/web/data")
            .display()
            .to_string();
        let file_cache = FileCache::new(&config);
        let storage = DomainStorage::init(&config.file_dir, file_cache).unwrap();
        let result = storage.get_domain_info().unwrap();

        println!("{:?}", result);
    }

    #[ignore]
    #[test]
    fn test_file_read() {
        let mut file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .write(true)
            .open("/tmp/cde.txt")
            .unwrap();
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        assert_eq!(&text, "");
        //writeln!(file, "{}", "abc").unwrap();
    }
}
