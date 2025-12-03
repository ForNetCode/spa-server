use crate::Config;
use anyhow::anyhow;
use entity::request::{
    DeleteDomainVersionOption, DomainWithOptVersionOption, DomainWithVersionOption,
    GetDomainOption, UpdateUploadingStatusOption,
};
use entity::storage::{CertInfo, DomainInfo, ShortMetaData, UploadDomainPosition};
use reqwest::{StatusCode, header, multipart};
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Clone)]
pub struct API {
    async_client: reqwest::Client,
    address: String,
}

macro_rules! handle {
    ($resp:ident) => {
        if $resp.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(anyhow!($resp.text().await?))
        }
    };
}
macro_rules! string_resp {
    ($resp:ident) => {
        if $resp.status() == StatusCode::OK {
            Ok($resp.text().await?)
        } else {
            Err(anyhow!($resp.text().await?))
        }
    };
}
macro_rules! json_resp {
    ($resp:ident) => {
        if $resp.status() == StatusCode::OK {
            Ok($resp.json::<Value>().await?)
        } else {
            Err(anyhow!($resp.text().await?))
        }
    };
    ($resp:ident, $t:ty) => {
        if $resp.status() == StatusCode::OK {
            let resp = $resp.json::<$t>().await?;
            tracing::debug!("{:?}", &resp);
            Ok(resp)
        } else {
            Err(anyhow!($resp.text().await?))
        }
    };
}

impl API {
    pub fn new(conf: &Config) -> anyhow::Result<API> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(format!("Bearer {}", &conf.server.auth_token).as_str())
                .unwrap(),
        );
        let mut builder = reqwest::Client::builder();
        builder = builder.default_headers(headers);
        let async_client = builder.build()?;
        Ok(API {
            async_client,
            address: conf.server.address.clone(),
        })
    }

    fn url(&self, uri: &str) -> String {
        format!("{}/{}", self.address, uri)
    }

    pub async fn get_domain_info(&self, domain: Option<String>) -> anyhow::Result<Vec<DomainInfo>> {
        let param = GetDomainOption { domain };
        let req = self.async_client.get(self.url("status")).query(&param);
        let resp = req.send().await?;
        json_resp!(resp, Vec<DomainInfo>)
    }

    pub async fn change_uploading_status(
        &self,
        param: UpdateUploadingStatusOption,
    ) -> anyhow::Result<()> {
        let resp = self
            .async_client
            .post(self.url("files/upload_status"))
            .json(&param)
            .send()
            .await?;
        handle!(resp)
    }

    pub async fn release_domain_version(
        &self,
        domain: String,
        version: Option<u32>,
    ) -> anyhow::Result<String> {
        let resp = self
            .async_client
            .post(self.url("update_version"))
            .json(&DomainWithOptVersionOption { domain, version })
            .send()
            .await?;
        string_resp!(resp)
    }

    pub async fn remove_files(
        &self,
        domain: Option<String>,
        max_reserve: Option<u32>,
    ) -> anyhow::Result<()> {
        let resp = self
            .async_client
            .post(self.url("files/delete"))
            .json(&DeleteDomainVersionOption {
                domain,
                max_reserve,
            })
            .send()
            .await?;
        handle!(resp)
    }

    pub async fn revoke_version(&self, domain: String, version: u32) -> anyhow::Result<()> {
        let resp = self
            .async_client
            .post(self.url("files/revoke_version"))
            .json(&DomainWithVersionOption { domain, version })
            .send()
            .await?;
        handle!(resp)
    }

    //TODO: use thiserror instead of anyhow
    pub async fn upload_file<T: Into<Cow<'static, str>>>(
        &self,
        domain: T,
        version: T,
        key: T,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        // let name = path.file_name().unwrap().to_os_string();
        // let name = name.to_str().unwrap().to_string();
        // let file = tokio::fs::File::open(path).await?;
        // let len = file.metadata().await?.len();
        // let file_part = multipart::Part::stream_with_length(file, len).file_name(name);
        let file_part = multipart::Part::file(&path).await?;
        let form = multipart::Form::new().part("file", file_part);

        let resp = self
            .async_client
            .post(self.url("file/upload"))
            .query(&[
                ("domain", domain.into()),
                ("version", version.into()),
                ("path", key.into()),
            ])
            .multipart(form)
            .send()
            .await?;
        if resp.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(anyhow!(resp.text().await?))
        }
    }

    pub async fn get_file_metadata(
        &self,
        domain: &str,
        version: u32,
    ) -> anyhow::Result<Vec<ShortMetaData>> {
        let resp = self
            .async_client
            .get(self.url("files/metadata"))
            .query(&[("domain", domain), ("version", &version.to_string())])
            .send()
            .await?;
        json_resp!(resp, Vec<ShortMetaData>)
    }

    pub async fn get_upload_position(&self, domain: &str) -> anyhow::Result<UploadDomainPosition> {
        let resp = self
            .async_client
            .get(self.url("upload/position"))
            .query(&[("domain", domain), ("format", "Json")])
            .send()
            .await?;
        json_resp!(resp, UploadDomainPosition)
    }

    pub async fn get_acme_cert_info(
        &self,
        domain: Option<String>,
    ) -> anyhow::Result<Vec<CertInfo>> {
        let resp = self
            .async_client
            .get(self.url("cert/acme"))
            .query(&GetDomainOption { domain })
            .send()
            .await?;
        json_resp!(resp, Vec<CertInfo>)
    }
}
#[cfg(test)]
mod test {
    use crate::LOCAL_HOST;
    use crate::api::API;
    use entity::request::UpdateUploadingStatusOption;
    use entity::storage::UploadingStatus;

    fn get_api() -> API {
        let config = crate::config::test::default_local_config().unwrap();
        API::new(&config).unwrap()
    }
    #[ignore]
    #[tokio::test]
    async fn get_domain_info() {
        let api = get_api();
        let response = api.get_domain_info(None).await.unwrap();
        println!("{:?}", response);
    }

    #[ignore]
    #[tokio::test]
    async fn get_file_metadata() {
        let api = get_api();
        let r = api.get_file_metadata(LOCAL_HOST, 1).await;
        println!("{:?}", r);
    }
    #[ignore]
    #[tokio::test]
    async fn update_upload_status() {
        let api = get_api();
        let r = api
            .change_uploading_status(UpdateUploadingStatusOption {
                domain: "www.baidu.com".to_owned(),
                version: 1,
                status: UploadingStatus::Finish,
            })
            .await;
        println!("{:?}", r);
    }
}
