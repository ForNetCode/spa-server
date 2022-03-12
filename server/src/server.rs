use hyper::service::{make_service_fn, service_fn};
use std::net::SocketAddr;
use std::str::FromStr;
//use tracing_log::LogTracer;
use crate::config::Config;
use crate::static_file_service::echo;

pub struct Server {
    conf: Config,
}

impl Server {
    pub fn new() -> Self {
        let conf = Config::load();
        Server { conf }
    }
    pub async fn run(&self) -> anyhow::Result<()> {
        let addr =
            SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();

        let server = hyper::Server::bind(&addr)
            .serve(make_service_fn(|_| async {
                Ok::<_, hyper::Error>(service_fn(echo))
            }))
            .await;
        Ok(server?)
    }
}
