use crate::admin_server::request::{GetDomainOption, GetDomainPathOption, UpdateDomainOption};
use crate::config::AdminConfig;
use crate::domain_storage::{DomainStorage, URI_REGEX};
use crate::with;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use warp::http::StatusCode;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

pub struct AdminServer {
    conf: AdminConfig,
    domain_storage: Arc<DomainStorage>,
}

impl AdminServer {
    pub fn new(conf: AdminConfig, domain_storage: Arc<DomainStorage>) -> Self {
        AdminServer {
            conf,
            domain_storage,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let bind_address =
            SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();
        let routes = warp::get()
            .and(self.auth())
            .and(
                warp::path("status")
                    .and(warp::query::<GetDomainOption>())
                    .and(with(self.domain_storage.clone()))
                    .and_then(service::get_domain_info)
                    .or(self.get_domain_upload_path()),
            )
            .or(warp::post()
                .and(self.auth())
                .and(self.update_domain_version()));

        warp::serve(routes).run(bind_address).await;
        Ok(())
    }

    fn auth(&self) -> impl Filter<Extract = (), Error = Rejection> + Clone {
        // this will not trigger memory leak, but be careful to use it
        warp::header::exact(
            "authorization",
            Box::leak(format!("Bearer {}", &self.conf.token).into_boxed_str()),
        )
    }

    fn get_domain_upload_path(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        async fn handle(
            option: GetDomainPathOption,
            storage: Arc<DomainStorage>,
        ) -> Result<Response, Infallible> {
            if URI_REGEX.is_match(&option.domain) {
                Ok(storage
                    .get_new_upload_path(&option.domain)
                    .await
                    .to_string_lossy()
                    .to_string()
                    .into_response())
            } else {
                Ok(StatusCode::BAD_REQUEST.into_response())
            }
        }
        warp::path!("upload" / "path")
            .and(warp::query::<GetDomainPathOption>())
            .and(with(self.domain_storage.clone()))
            .and_then(handle)
    }

    fn update_domain_version(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        async fn handle(
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
                    info!("{}", &text);
                    Ok(text.into_response())
                }
                Err(_) => Ok(StatusCode::NOT_FOUND.into_response()),
            }
        }
        warp::path("update_version")
            .and(warp::path::end())
            .and(
                warp::body::content_length_limit(1024 * 16)
                    .and(warp::body::json::<UpdateDomainOption>()),
            )
            .and(with(self.domain_storage.clone()))
            .and_then(handle)
    }
}

mod service {
    use crate::admin_server::request::GetDomainOption;
    use crate::domain_storage::DomainStorage;
    use std::convert::Infallible;
    use std::sync::Arc;
    use warp::http::StatusCode;
    use warp::reply::Response;
    use warp::Reply;

    pub async fn get_domain_info(
        option: GetDomainOption,
        storage: Arc<DomainStorage>,
    ) -> Result<Response, Infallible> {
        let domain_info = storage.get_domain_info().await;
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
// TODO: the code structure is friendly with Unit Test, need refactor it.
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
