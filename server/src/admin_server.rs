use crate::admin_server::request::{
    DomainWithVersionOption, GetDomainOption, GetDomainPathOption, UpdateUploadingStatusOption,
};
use crate::config::AdminConfig;
use crate::domain_storage::DomainStorage;
use crate::hot_reload::HotReloadManager;
use crate::with;
use hyper::{Body, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use warp::multipart::FormData;
use warp::reply::Response;
use warp::{Filter, Rejection};

pub struct AdminServer {
    conf: AdminConfig,
    domain_storage: Arc<DomainStorage>,
    reload_manager: HotReloadManager,
}

impl AdminServer {
    pub fn new(
        conf: &AdminConfig,
        domain_storage: Arc<DomainStorage>,
        reload_manager: HotReloadManager,
    ) -> Self {
        AdminServer {
            conf: conf.clone(),
            domain_storage,
            reload_manager,
        }
    }

    fn routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        self.auth().and(
            (warp::get().and(
                self.get_domain_info()
                    .or(self.get_domain_upload_path())
                    .or(self.get_files_metadata()),
            ))
            .or(warp::post().and(
                self.update_domain_version()
                    .or(self.reload_server())
                    .or(self.change_upload_status())
                    .or(self.upload_file()),
            )),
        )
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let bind_address =
            SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();
        warp::serve(self.routes()).run(bind_address).await;
        Ok(())
    }

    fn auth(&self) -> impl Filter<Extract = (), Error = Rejection> + Clone {
        // this will not trigger memory leak, be careful to use it
        warp::header::exact(
            "authorization",
            Box::leak(format!("Bearer {}", &self.conf.token).into_boxed_str()),
        )
    }

    fn get_domain_info(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        warp::path("status")
            .and(warp::query::<GetDomainOption>())
            .and(with(self.domain_storage.clone()))
            .and_then(service::get_domain_info)
    }

    fn get_domain_upload_path(
        &self,
    ) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone {
        warp::path!("upload" / "path")
            .and(warp::query::<GetDomainPathOption>())
            .and(with(self.domain_storage.clone()))
            .map(service::get_domain_upload_path)
    }

    fn update_domain_version(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        warp::path("update_version")
            .and(warp::path::end())
            .and(
                warp::body::content_length_limit(1024 * 16)
                    .and(warp::body::json::<DomainWithVersionOption>()),
            )
            .and(with(self.domain_storage.clone()))
            .and_then(service::update_domain_version)
    }

    fn reload_server(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        let admin_config = Arc::new(self.conf.clone());
        let reload_manager = Arc::new(self.reload_manager.clone());

        warp::path("reload")
            .and(warp::path::end())
            .and(with(reload_manager))
            .and(with(admin_config))
            .and_then(service::reload_server)
    }

    fn change_upload_status(
        &self,
    ) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone {
        warp::path!("files" / "upload_status")
            .and(with(self.domain_storage.clone()))
            .and(
                warp::body::content_length_limit(1024 * 16)
                    .and(warp::body::json::<UpdateUploadingStatusOption>()),
            )
            .map(service::change_upload_status)
    }

    fn upload_file(&self) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        async fn handler(
            form: FormData,
            storage: Arc<DomainStorage>,
        ) -> Result<Response, Infallible> {
            let resp = service::update_file(form, storage)
                .await
                .unwrap_or_else(|e| {
                    let mut resp = Response::new(Body::from(e.to_string()));
                    *resp.status_mut() = StatusCode::BAD_REQUEST;
                    resp
                });
            Ok(resp)
        }
        warp::path!("file" / "upload")
            .and(warp::path::end())
            .and(warp::multipart::form().max_length(self.conf.max_upload_size))
            .and(with(self.domain_storage.clone()))
            .and_then(handler)
    }

    fn get_files_metadata(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        warp::path!("files" / "metadata")
            .and(with(self.domain_storage.clone()))
            .and(warp::query::<DomainWithVersionOption>())
            .map(service::get_files_metadata)
    }
}

pub mod service {
    use crate::admin_server::request::{
        DomainWithVersionOption, GetDomainOption, GetDomainPathOption, UpdateUploadingStatusOption,
    };
    use crate::domain_storage::{DomainStorage, URI_REGEX};
    use crate::{AdminConfig, HotReloadManager};
    use anyhow::anyhow;
    use bytes::{Buf, Bytes};
    use futures_util::TryStreamExt;
    use hyper::Body;
    use if_chain::if_chain;
    use std::convert::Infallible;
    use std::sync::Arc;
    use warp::http::StatusCode;
    use warp::multipart::{FormData, Part};
    use warp::reply::Response;
    use warp::Reply;

    pub(super) async fn get_domain_info(
        option: GetDomainOption,
        storage: Arc<DomainStorage>,
    ) -> Result<Response, Infallible> {
        let domain_info = storage.get_domain_info();
        match option.domain {
            Some(domain) => {
                if let Some(data) = domain_info.iter().find(|x| x.domain == domain) {
                    return Ok(warp::reply::json(data).into_response());
                } else {
                    return Ok(StatusCode::NOT_FOUND.into_response());
                }
            }
            None => Ok(warp::reply::json(&domain_info).into_response()),
        }
    }

    pub(super) async fn update_domain_version(
        option: DomainWithVersionOption,
        storage: Arc<DomainStorage>,
    ) -> Result<Response, Infallible> {
        match storage
            .upload_domain_with_version(option.domain.clone(), option.version)
            .await
        {
            Ok(_) => {
                let text = format!("domain:{} has changed to {}", option.domain, option.version);
                tracing::info!("{}", &text);
                Ok(text.into_response())
            }
            Err(_) => Ok(StatusCode::NOT_FOUND.into_response()),
        }
    }

    pub(super) fn get_domain_upload_path(
        option: GetDomainPathOption,
        storage: Arc<DomainStorage>,
    ) -> Response {
        if URI_REGEX.is_match(&option.domain) {
            storage
                .get_new_upload_path(&option.domain)
                .to_string_lossy()
                .to_string()
                .into_response()
        } else {
            StatusCode::BAD_REQUEST.into_response()
        }
    }

    pub(super) async fn reload_server(
        reload_manager: Arc<HotReloadManager>,
        admin_config: Arc<AdminConfig>,
    ) -> Result<Response, Infallible> {
        let resp = match crate::reload_server(&admin_config, reload_manager.as_ref()).await {
            Ok(_) => Response::default(),
            Err(e) => {
                let mut resp = Response::new(Body::from(format!("error:{}", e)));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                resp
            }
        };
        Ok(resp)
    }

    pub(super) fn change_upload_status(
        storage: Arc<DomainStorage>,
        param: UpdateUploadingStatusOption,
    ) -> Response {
        match storage.update_uploading_status(param.domain, param.version, param.status) {
            Ok(_) => Response::default(),
            Err(e) => {
                let mut resp = Response::new(Body::from(e.to_string()));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                resp
            }
        }
    }

    pub(super) async fn update_file(
        form: FormData,
        storage: Arc<DomainStorage>,
    ) -> anyhow::Result<Response> {
        let mut parts: Vec<Part> = form.try_collect().await?;

        let mut file_buf: Option<Bytes> = None;
        let mut path: Option<String> = None;
        let mut version: Option<u32> = None;
        let mut domain: Option<String> = None;
        // this convert cost so much code, it should has more nice way to do data convert.
        for p in parts.iter_mut() {
            let name = p.name();
            if name == "file" {
                file_buf = p
                    .data()
                    .await
                    .map(|x| {
                        x.map(|mut x| {
                            let i = x.remaining();
                            (&mut x).copy_to_bytes(i)
                        })
                        .ok()
                    })
                    .flatten();
            } else if name == "path" {
                path = p
                    .data()
                    .await
                    .map(|x| {
                        x.map(|mut buf| {
                            let i = buf.remaining();
                            String::from_utf8((&mut buf).copy_to_bytes(i).to_vec()).ok()
                        })
                        .ok()
                        .flatten()
                    })
                    .flatten();
            } else if name == "version" {
                version = p
                    .data()
                    .await
                    .map(|x| {
                        x.map(|mut buf| {
                            let i = buf.remaining();
                            String::from_utf8((&mut buf).copy_to_bytes(i).to_vec())
                                .ok()
                                .map(|x| x.parse::<u32>().ok())
                                .flatten()
                        })
                        .ok()
                        .flatten()
                    })
                    .flatten();
            } else if name == "domain" {
                domain = p
                    .data()
                    .await
                    .map(|x| {
                        x.map(|mut buf| {
                            let i = buf.remaining();
                            String::from_utf8((&mut buf).copy_to_bytes(i).to_vec()).ok()
                        })
                        .ok()
                        .flatten()
                    })
                    .flatten();
            }
        }
        tracing::debug!("uploading file: {:?}, {:?}, {:?}", domain, version, path);
        if_chain! {
            if let Some(_path) = path;
            if let Some(_version) = version;
            if let Some(_domain) = domain;
            if let Some(_file_buf) = file_buf;
            then {
                storage.save_file(_domain, _version, _path, _file_buf)?;
                Ok(Response::default())
            } else {
                Err(anyhow!("bad param"))
            }
        }
    }

    pub(super) fn get_files_metadata(
        storage: Arc<DomainStorage>,
        query: DomainWithVersionOption,
    ) -> Response {
        match storage.get_files_metadata(query.domain, query.version) {
            Ok(data) => warp::reply::json(&data).into_response(),
            Err(err) => {
                let mut resp = Response::new(Body::from(err.to_string()));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                resp
            }
        }
    }
}

pub mod request {
    use crate::domain_storage::UploadingStatus;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub struct GetDomainOption {
        pub domain: Option<String>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct GetDomainPathOption {
        pub domain: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct DomainWithVersionOption {
        pub domain: String,
        pub version: u32,
    }
    #[derive(Deserialize, Serialize)]
    pub struct UpdateUploadingStatusOption {
        pub domain: String,
        pub version: u32,
        pub status: UploadingStatus,
    }
}

// TODO: the code structure is not friendly with Unit Test, need refactor it.
#[cfg(test)]
mod test {
    use crate::admin_server::request::DomainWithVersionOption;
    use warp::test::request;

    #[tokio::test]
    async fn update_domain_version_test() {
        let body = DomainWithVersionOption {
            domain: "self.noti.link".to_string(),
            version: 1,
        };
        /*
        let resp = request()
            .method("POST")
            .path("/update_version")
            .json(&body)
            .reply(&api)
            .await;*/
    }
}
