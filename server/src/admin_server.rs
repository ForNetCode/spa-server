use crate::admin_server::request::{GetDomainOption, GetDomainPathOption, UpdateDomainOption};
use crate::config::AdminConfig;
use crate::domain_storage::{DomainStorage};
use crate::{with};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use warp::{Filter, Rejection};
use warp::reply::Response;
use crate::hot_reload::HotReloadManager;

pub struct AdminServer {
    conf: AdminConfig,
    domain_storage: Arc<DomainStorage>,
    reload_manager: HotReloadManager,
}

impl AdminServer {
    pub fn new(conf: &AdminConfig, domain_storage: Arc<DomainStorage>, reload_manager: HotReloadManager) -> Self {
        AdminServer {
            conf: conf.clone(),
            domain_storage,
            reload_manager,
        }
    }

    fn routes(&self) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        self.auth()
            .and(
                (warp::get().and(self.get_domain_info().or(self.get_domain_upload_path())))
            .or(warp::post().and(self.update_domain_version()).or(self.reload_server())))
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let bind_address =
            SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();
        warp::serve(self.routes()).run(bind_address).await;
        Ok(())
    }

    fn auth(&self) -> impl Filter<Extract=(), Error=Rejection> + Clone {
        // this will not trigger memory leak, be careful to use it
        warp::header::exact(
            "authorization",
            Box::leak(format!("Bearer {}", &self.conf.token).into_boxed_str()),
        )
    }

    fn get_domain_info(&self) -> impl Filter<Extract=(impl warp::Reply, ), Error=Rejection> + Clone {
        warp::path("status")
            .and(warp::query::<GetDomainOption>())
            .and(with(self.domain_storage.clone()))
            .and_then(service::get_domain_info)
    }

    fn get_domain_upload_path(
        &self,
    ) -> impl Filter<Extract=(Response, ), Error=Rejection> + Clone {
        warp::path!("upload" / "path")
            .and(warp::query::<GetDomainPathOption>())
            .and(with(self.domain_storage.clone()))
            .map(service::get_domain_upload_path)
    }

    fn update_domain_version(
        &self,
    ) -> impl Filter<Extract=(impl warp::Reply, ), Error=Rejection> + Clone {
        warp::path("update_version")
            .and(warp::path::end())
            .and(
                warp::body::content_length_limit(1024 * 16)
                    .and(warp::body::json::<UpdateDomainOption>()),
            )
            .and(with(self.domain_storage.clone()))
            .and_then(service::update_domain_version)
    }

    fn reload_server(&self) -> impl Filter<Extract=(impl warp::Reply, ), Error=Rejection> + Clone {
        let admin_config = Arc::new(self.conf.clone());
        let reload_manager = Arc::new(self.reload_manager.clone());

        warp::path("reload").and(warp::path::end())
            .and(with(reload_manager))
            .and(with(admin_config))
            .and_then(service::reload_server)
    }
}

pub mod service {
    use crate::admin_server::request::{GetDomainOption, GetDomainPathOption, UpdateDomainOption};
    use crate::domain_storage::{DomainStorage, URI_REGEX};
    use std::convert::Infallible;
    use std::sync::Arc;
    use hyper::Body;
    use warp::http::StatusCode;
    use warp::reply::Response;
    use warp::{Reply};
    use crate::{AdminConfig, HotReloadManager};

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
        option: UpdateDomainOption,
        storage: Arc<DomainStorage>,
    ) -> Result<Response, Infallible> {
        match storage
            .upload_domain_with_version(option.domain.clone(), option.version)
            .await
        {
            Ok(_) => {
                let text =
                    format!("domain:{} has changed to {}", option.domain, option.version);
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
            Ok(_) => {
                Response::default()
            }
            Err(e) => {
                let mut resp = Response::new(Body::from(format!("error:{}", e)));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                resp
            }
        };
        Ok(resp)
    }
}

mod request {
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
    pub struct UpdateDomainOption {
        pub domain: String,
        pub version: i32,
    }
}

// TODO: the code structure is not friendly with Unit Test, need refactor it.
#[cfg(test)]
mod test {
    use crate::admin_server::request::UpdateDomainOption;
    use warp::test::request;

    #[tokio::test]
    async fn update_domain_version_test() {
        let body = UpdateDomainOption {
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
