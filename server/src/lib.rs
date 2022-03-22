#![allow(dead_code)]
#![allow(unused_variables)]

pub mod server;

pub mod admin_server;
pub mod config;
pub mod domain_storage;
pub mod file_cache;
pub mod tls;
pub mod hot_reload;

mod redirect_https;
mod static_file_filter;

// utils
use crate::admin_server::AdminServer;
use crate::config::{AdminConfig, Config};
use crate::domain_storage::DomainStorage;
use crate::file_cache::FileCache;
use futures::future::join;
pub use server::Server;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use warp::Filter;
use crate::hot_reload::{HotReloadManager, HotReloadState};

pub fn with<T: Send + Sync>(
    d: Arc<T>,
) -> impl Filter<Extract = (Arc<T>,), Error = Infallible> + Clone {
    warp::any().map(move || d.clone())
}

async fn run_admin_server(
    config: &Option<AdminConfig>,
    storage: &Arc<DomainStorage>,
    reload_manager:HotReloadManager,
) -> anyhow::Result<()> {
    if let Some(admin_config) = config {
        tracing::info!("admin server enabled.");
        let admin_server = AdminServer::new(&admin_config, storage.clone(), reload_manager);
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

pub async fn reload_server(admin_config:&AdminConfig, reload_manager:&HotReloadManager) -> anyhow::Result<()> {
    // TODO:
    // check: if port can bind.
    // check: if cert file is ok.
    let config = Config::load()?;
    if config.admin_config.as_ref() == Some(admin_config) {
        let cache = FileCache::new(config.cache.clone());
        let domain_storage = Arc::new(DomainStorage::init(&config.file_dir.clone(), cache)?);
        let (state, http_rx, https_rx) = HotReloadState::init(&config);
        let server = Server::new(config.clone(), domain_storage.clone());
        tokio::task::spawn(async move{server.run(http_rx, https_rx).await});
        // sleep 500
        tokio::time::sleep(Duration::from_millis(500)).await;
        reload_manager.reload(state).await?;
    }
    Ok(())
}
pub async fn run_server() -> anyhow::Result<()> {
    let (config, domain_storage) = load_config_and_cache().expect("prepare config and cache file");
    let server = Server::new(config.clone(), domain_storage.clone());

    let (reload_manager, http_rx, https_rx) = HotReloadManager::init(&config);
    let (_ret1,_ret2) = join(
        server.run(http_rx, https_rx),
        run_admin_server(&config.admin_config, &domain_storage,reload_manager),
    )
    .await;
    _ret1?;
    _ret2?;
    Ok(())
}
