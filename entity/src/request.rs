use crate::storage::UploadingStatus;
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetDomainOption {
    pub domain: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Default, ToSchema)]
pub enum GetDomainPositionFormat {
    #[default]
    Path,
    Json,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct GetDomainPositionOption {
    pub domain: String,
    //#[serde(default="crate::admin_server::request::GetDomainPositionFormat::Path")]
    #[serde(default)]
    pub format: GetDomainPositionFormat,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct UploadFileOption {
    pub domain: String,
    pub version: u32,
    pub path: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct DomainWithVersionOption {
    pub domain: String,
    pub version: u32,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct DomainWithOptVersionOption {
    pub domain: String,
    pub version: Option<u32>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateUploadingStatusOption {
    pub domain: String,
    pub version: u32,
    pub status: UploadingStatus,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct DeleteDomainVersionOption {
    pub domain: Option<String>,
    pub max_reserve: Option<u32>,
}
