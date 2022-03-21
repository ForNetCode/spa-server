#![allow(dead_code)]
#![allow(unused_variables)]

pub mod server;

pub mod admin_server;
pub mod config;
pub mod domain_storage;
pub mod file_cache;
mod redirect_https;
mod static_file_filter;
pub mod tls;

// utils
use crate::admin_server::AdminServer;
use crate::config::{AdminConfig, Config};
use crate::domain_storage::DomainStorage;
use crate::file_cache::FileCache;
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

async fn run_admin_server(
    config: &Option<AdminConfig>,
    storage: &Arc<DomainStorage>,
) -> anyhow::Result<()> {
    if let Some(admin_config) = config {
        tracing::info!("admin server enabled.");
        let admin_server = AdminServer::new(admin_config.clone(), storage.clone());
        return admin_server.run().await;
    } else {
        tracing::info!("admin server disabled.");
    }
    Ok(())
}

fn load_config_and_cache() -> anyhow::Result<(Config, Arc<DomainStorage>)> {
    let config = Config::load()?;
    tracing::debug!("config load:{:?}", &config);
    let cache = FileCache::new(config.cache.clone());
    let domain_storage = Arc::new(DomainStorage::init(&config.file_dir.clone(), cache)?);
    Ok((config, domain_storage))
}

pub async fn run_server() -> anyhow::Result<()> {
    let (config, domain_storage) = load_config_and_cache().expect("prepare config and cache file");
    let server = Server::new(config.clone(), domain_storage.clone());

    let _ret = join(
        server.run(),
        run_admin_server(&config.admin_config, &domain_storage),
    )
    .await;
    Ok(())
}
