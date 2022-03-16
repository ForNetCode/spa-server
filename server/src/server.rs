use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use crate::config::Config;
use crate::domain_storage::DomainStorage;
use crate::join;
use crate::static_file_filter::static_file_filter;
use futures::future::OptionFuture;

pub struct Server {
    conf: Config,
    storage: Arc<DomainStorage>,
}

impl Server {
    pub fn new(conf: Config, storage: Arc<DomainStorage>) -> Self {
        Server { conf, storage }
    }
    pub async fn run(&self) -> anyhow::Result<()> {
        let filter = static_file_filter(self.storage.clone());

        //disable http server
        if self.conf.port <= 0 && self.conf.https.is_none() {
            panic!("should set http or https server config");
        }
        let https_server: OptionFuture<_> = if let Some(config) = &self.conf.https {
            let bind_address =
                SocketAddr::from_str(&format!("{}:{}", &config.addr, &config.port)).unwrap();

            Some(
                warp::serve(filter.clone())
                    .tls()
                    .cert_path(&config.public)
                    .key_path(&config.private)
                    .run(bind_address),
            )
        } else {
            None
        }
        .into();
        let http_server: OptionFuture<_> = (if self.conf.port > 0 {
            let bind_address =
                SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();
            Some(warp::serve(filter).run(bind_address))
        } else {
            None
        })
        .into();
        let _re = join(https_server, http_server).await;
        Ok(())
    }
}
