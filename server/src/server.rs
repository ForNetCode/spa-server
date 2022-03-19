use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

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

    async fn start_https_server(&self) -> anyhow::Result<()> {
        if let Some(config) = &self.conf.https {
            let bind_address =
                SocketAddr::from_str(&format!("{}:{}", &config.addr, &config.port)).unwrap();
            let filter = static_file_filter(self.storage.clone());
            warp::serve(filter.clone())
                .tls()
                .cert_path(&config.public)
                .key_path(&config.private)
                .run(bind_address)
                .await;
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
                .clone()
                .map(|x| x.http_redirect_to_https.to_owned())
                .flatten()
                .is_some()
            {
                hyper_redirect_server(bind_address, self.storage.clone()).await?;
            } else {
                let filter = static_file_filter(self.storage.clone());
                warp::serve(filter).run(bind_address).await;
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
