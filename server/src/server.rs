use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use crate::config::Config;
use crate::domain_storage::DomainStorage;
use crate::static_file_filter::static_file_filter;

pub struct Server {
    conf: Config,
    storage: Arc<DomainStorage>,
}

impl Server {
    pub fn new(conf: Config, storage: Arc<DomainStorage>) -> Self {
        Server { conf, storage }
    }
    pub async fn run(&self) -> anyhow::Result<()> {
        let bind_address =
            SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();
        let filter = static_file_filter(self.storage.clone());
        let server = warp::serve(filter);
        // only support one ssl
        if let Some(ssl_config) = self.conf.ssl.0.get(0) {
            server
                .tls()
                .cert_path(&ssl_config.public)
                .key_path(&ssl_config.private)
                .run(bind_address)
                .await;
        } else {
            server.run(bind_address).await;
        }
        Ok(())
    }
}
