use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use crate::config::Config;
use crate::domain_storage::DomainStorage;
use crate::static_file_filter::static_file_filter;

pub struct Server {
    conf: Config,
}

impl Server {
    pub fn new() -> Self {
        let conf = Config::load();
        info!("file_dir is {}", &conf.file_dir);
        Server { conf }
    }
    pub async fn run(&self) -> anyhow::Result<()> {
        let bind_address =
            SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();
        let domain_storage = Arc::new(DomainStorage::init(&self.conf.file_dir).unwrap());
        // let server = hyper::Server::bind(&bind_address)
        //     .serve(make_service_fn(|_| {
        //         let domain_storage = domain_storage.clone();
        //         async move {
        //             Ok::<_, hyper::Error>(service_fn(move |_req| {
        //                 static_file_service(domain_storage.clone(), _req)
        //             }))
        //         }
        //     }))
        //     .await;
        let filter = static_file_filter(domain_storage);
        warp::serve(filter).run(bind_address).await;
        Ok(())
    }
}
