use serde::{Deserialize, Serialize};
use crate::storage::UploadingStatus;

#[derive(Deserialize, Serialize)]
pub struct GetDomainOption {
    pub domain: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Default)]
pub enum GetDomainPositionFormat {
    #[default]
    Path,
    Json,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetDomainPositionOption {
    pub domain: String,
    //#[serde(default="crate::admin_server::request::GetDomainPositionFormat::Path")]
    #[serde(default)]
    pub format: GetDomainPositionFormat,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UploadFileOption {
    pub domain: String,
    pub version: u32,
    pub path: String,
}

#[derive(Deserialize, Serialize)]
pub struct DomainWithVersionOption {
    pub domain: String,
    pub version: u32,
}

#[derive(Deserialize, Serialize)]
pub struct DomainWithOptVersionOption {
    pub domain: String,
    pub version: Option<u32>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateUploadingStatusOption {
    pub domain: String,
    pub version: u32,
    pub status: UploadingStatus,
}

#[derive(Deserialize, Serialize)]
pub struct DeleteDomainVersionOption {
    pub domain: Option<String>,
    pub max_reserve: Option<u32>,
}
