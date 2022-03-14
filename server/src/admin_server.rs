use crate::admin_server::request::{GetDomainOption, GetDomainPathOption};
use crate::config::AdminConfig;
use crate::domain_storage::{DomainStorage, URI_REGEX};
use crate::with;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use warp::http::StatusCode;
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
    pub async fn run(&self) {
        let bind_address =
            SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();
        info!("admin server bind {}", &bind_address);
        let routes = warp::get().and(
            warp::path("status")
                .and(warp::query::<GetDomainOption>())
                .and(with(self.domain_storage.clone()))
                .map(service::get_domain_info)
                .or(self.get_domain_upload_path()),
        );

        warp::serve(routes).run(bind_address).await;
    }
    fn get_domain_upload_path(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        warp::path("upload/path")
            .and(warp::query::<GetDomainPathOption>())
            .and(with(self.domain_storage.clone()))
            .map(|option: GetDomainPathOption, storage: Arc<DomainStorage>| {
                if URI_REGEX.is_match(&option.domain) {
                    storage
                        .get_new_upload_path(&option.domain)
                        .to_string_lossy()
                        .to_string()
                        .into_response()
                } else {
                    StatusCode::BAD_REQUEST.into_response()
                }
            })
    }
}
mod service {
    use crate::admin_server::request::GetDomainOption;
    use crate::domain_storage::DomainStorage;
    use std::sync::Arc;
    use warp::http::StatusCode;

    pub fn get_domain_info(
        option: GetDomainOption,
        storage: Arc<DomainStorage>,
    ) -> Box<dyn warp::Reply> {
        let domain_info = storage.get_domain_info();
        match option.domain {
            Some(domain) => {
                if let Some(data) = domain_info.iter().find(|x| x.domain == domain) {
                    return Box::new(warp::reply::json(data));
                } else {
                    return Box::new(StatusCode::NOT_FOUND);
                }
            }
            None => Box::new(warp::reply::json(&domain_info)),
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
}

mod model {}
