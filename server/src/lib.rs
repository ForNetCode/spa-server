#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate log;

pub mod server;

pub mod admin_server;
pub mod config;
pub mod domain_storage;
mod static_file_filter;

// utils
use crate::admin_server::AdminServer;
use crate::config::Config;
use crate::domain_storage::DomainStorage;
use futures::future::join;
pub use server::Server;
use std::convert::Infallible;
use std::sync::Arc;
use warp::Filter;

pub fn with<T: Send + Sync>(
    d: Arc<T>,
) -> impl Filter<Extract = (Arc<T>,), Error = Infallible> + Clone {
    warp::any().map(move || d.clone())
}

pub async fn run_server() -> anyhow::Result<()> {
    let config = Config::load();
    let domain_storage = Arc::new(DomainStorage::init(&config.file_dir.clone()).unwrap());
    let server = Server::new(config.clone(), domain_storage.clone());

    if let Some(admin_config) = config.admin_config {
        info!("admin server enabled.");
        let admin_server = AdminServer::new(admin_config, domain_storage.clone());
        let _ret = join(server.run(), admin_server.run()).await;
    } else {
        info!("admin server disabled.");
        server.run().await?;
    }
    Ok(())
}
