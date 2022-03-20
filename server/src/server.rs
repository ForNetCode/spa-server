use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use warp::cors::Cors;
use warp::Filter;

use crate::config::Config;
use crate::domain_storage::DomainStorage;
use crate::join;
use crate::redirect_https::hyper_redirect_server;
use crate::static_file_filter::static_file_filter;

pub struct Server {
    conf: Config,
    storage: Arc<DomainStorage>,
}

impl Server {
    pub fn new(conf: Config, storage: Arc<DomainStorage>) -> Self {
        Server { conf, storage }
    }

    fn get_cors_config(&self) -> Cors {
        warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "OPTION", "HEAD"])
            .max_age(3600)
            .build()
    }

    async fn start_https_server(&self) -> anyhow::Result<()> {
        if let Some(config) = &self.conf.https {
            let bind_address =
                SocketAddr::from_str(&format!("{}:{}", &config.addr, &config.port)).unwrap();
            let filter = static_file_filter(self.storage.clone());
            if self.conf.cors {
                tracing::debug!("enable CORS for https server");
                warp::serve(filter.with(self.get_cors_config()))
                    .tls()
                    .cert_path(&config.public)
                    .key_path(&config.private)
                    .run(bind_address)
                    .await;
            } else {
                warp::serve(filter)
                    .tls()
                    .cert_path(&config.public)
                    .key_path(&config.private)
                    .run(bind_address)
                    .await;
            }
        }
        Ok(())
    }

    async fn start_http_server(&self) -> anyhow::Result<()> {
        if self.conf.port > 0 {
            let bind_address =
                SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();
            if self
                .conf
                .https
                .as_ref()
                .map_or(false, |x| x.http_redirect_to_https)
            {
                hyper_redirect_server(bind_address, self.storage.clone()).await?;
            } else {
                let filter = static_file_filter(self.storage.clone());
                if self.conf.cors {
                    tracing::debug!("enable CORS for http server");
                    warp::serve(filter.with(self.get_cors_config()))
                        .run(bind_address)
                        .await;
                } else {
                    warp::serve(filter).run(bind_address).await;
                }
            }
        }
        Ok(())
    }
    pub async fn run(&self) -> anyhow::Result<()> {
        //disable http server
        if self.conf.port <= 0 && self.conf.https.is_none() {
            panic!("should set http or https server config");
        }
        let _re = join(self.start_http_server(), self.start_https_server()).await;
        Ok(())
    }
}
