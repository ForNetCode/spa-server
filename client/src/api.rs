use crate::Config;
use anyhow::anyhow;
use reqwest::{header, multipart, StatusCode};
use serde_json::Value;
use spa_server::admin_server::request::UpdateUploadingStatusOption;
use spa_server::domain_storage::ShortMetaData;
use std::borrow::Cow;
use std::path::PathBuf;

pub struct API {
    blocking_client: reqwest::blocking::Client,
    async_client: reqwest::Client,
    address: String,
}

macro_rules! handle {
    ($resp:ident) => {
        if $resp.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(anyhow!($resp.text()?))
        }
    };
}
macro_rules! json_resp {
    ($resp:ident) => {
        if $resp.status() == StatusCode::OK {
            Ok($resp.json::<Value>()?)
        } else {
            Err(anyhow!($resp.text()?))
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

        let mut builder = reqwest::blocking::Client::builder();
        builder = builder.default_headers(headers.clone());

        let blocking_client = builder.build()?;

        let mut builder = reqwest::Client::builder();
        builder = builder.default_headers(headers);
        let async_client = builder.build()?;
        Ok(API {
            blocking_client,
            async_client,
            address: conf.server.address.clone(),
        })
    }

    fn url(&self, uri: &str) -> String {
        format!("{}/{}", self.address, uri)
    }

    pub fn get_domain_info(&self, domain: Option<String>) -> anyhow::Result<Value> {
        let mut q = self.blocking_client.get(self.url("status"));
        if let Some(domain) = domain {
            q = q.query(&["domain", &domain])
        }
        let resp = q.send()?;
        json_resp!(resp)
    }

    pub fn change_uploading_status(
        &self,
        param: UpdateUploadingStatusOption,
    ) -> anyhow::Result<()> {
        let resp = self
            .blocking_client
            .post(self.url("files/upload_status"))
            .json(&param)
            .send()?;
        handle!(resp)
    }

    pub fn release_domain_version(&self, domain: String, version: u32) -> anyhow::Result<()> {
        //self.blocking_client
        let resp = self
            .blocking_client
            .post("update_version")
            .query(&[("domain", domain), ("version", version.to_string())])
            .send()?;
        handle!(resp)
    }

    pub fn reload_sap_server(&self) -> anyhow::Result<()> {
        let resp = self.blocking_client.post(self.url("reload")).send()?;
        handle!(resp)
    }

    pub async fn upload_file<T: Into<Cow<'static, str>>>(
        &self,
        domain: T,
        version: T,
        key: T,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let name = path.file_name().unwrap().to_os_string();
        let name = name.to_str().unwrap().to_string();
        let file = tokio::fs::File::open(path).await?;
        let len = file.metadata().await?.len();
        let file_part = multipart::Part::stream_with_length(file, len).file_name(name);
        let form = multipart::Form::new()
            .text("domain", domain)
            .text("version", version)
            .text("path", key)
            .part("file", file_part);

        let resp = self
            .async_client
            .post(self.url("file/upload"))
            .multipart(form)
            .send()
            .await?;
        if resp.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(anyhow!(resp.text().await?))
        }
    }

    pub fn get_file_metadata(
        &self,
        domain: &str,
        version: u32,
    ) -> anyhow::Result<Vec<ShortMetaData>> {
        let resp = self
            .blocking_client
            .get(self.url("files/metadata"))
            .query(&[("domain", domain), ("version", &version.to_string())])
            .send()?;
        if resp.status() == StatusCode::OK {
            let r = resp.json()?;
            Ok(r)
        } else {
            Err(anyhow!(resp.text()?))
        }
    }
}
#[cfg(test)]
mod test {
    use crate::api::API;
    use spa_server::admin_server::request::UpdateUploadingStatusOption;
    use spa_server::domain_storage::{DomainInfo, UploadingStatus};
    fn get_api() -> API {
        let config = crate::config::test::default_local_config().unwrap();
        API::new(&config).unwrap()
    }
    #[test]
    fn get_domain_info() {
        let api = get_api();
        let response = api.get_domain_info(None).unwrap();
        println!("{:?}", response);
    }

    #[test]
    fn get_file_metadata() {
        let api = get_api();
        let r = api.get_file_metadata("self.noti.link", 1);
        println!("{:?}", r);
        //api.upload_file("self.noti.link", &2.to_string(),PathBuf::new(""));
    }
    #[test]
    fn update_upload_status() {
        let api = get_api();
        let r = api.change_uploading_status(UpdateUploadingStatusOption {
            domain: "www.baidu.com".to_owned(),
            version: 1,
            status: UploadingStatus::Finish,
        });
        println!("{:?}", r);
    }
}
