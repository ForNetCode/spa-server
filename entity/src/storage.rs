use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

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
