//#![allow(dead_code)]
//#![allow(unused_variables)]

pub mod admin_server;
pub mod config;
pub mod domain_storage;
pub mod file_cache;
mod web_server;

pub mod service;

use crate::admin_server::AdminServer;
use crate::config::{AdminConfig, Config};
use crate::domain_storage::DomainStorage;
use crate::file_cache::FileCache;
use crate::service::ServiceConfig;
use crate::web_server::init_http_server;
use delay_timer::entity::DelayTimer;
use delay_timer::prelude::DelayTimerBuilder;
use futures_util::TryFutureExt;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;

async fn run_admin_server(
    config: &AdminConfig,
    storage: Arc<DomainStorage>,
    delay_timer: DelayTimer,
    host_alias: Arc<HashMap<String, String>>,
) -> anyhow::Result<()> {
    let admin_server = AdminServer::new(config, storage.clone(), delay_timer, host_alias);
    admin_server.run().await
}

pub async fn run_server() -> anyhow::Result<()> {
    let config = Config::load()?;
    tracing::debug!("config load:{:?}", &config);
    run_server_with_config(config).await
}

pub async fn run_server_with_config(config: Config) -> anyhow::Result<()> {
    let config = Arc::new(config);
    let cache = FileCache::new();
    let domain_storage = Arc::new(DomainStorage::init(&config.file_dir, cache)?);
    let service_config = Arc::new(ServiceConfig::new(&config));
    let host_alias = service_config.host_alias.clone();

    if let Some(admin_config) = &config.admin_config {
        tracing::info!("admin server enabled");
        let delay_timer = DelayTimerBuilder::default()
            .tokio_runtime_by_default()
            .build();

        let _ = tokio::join!(
            run_admin_server(
                admin_config,
                domain_storage.clone(),
                delay_timer,
                host_alias,
            )
            .map_err(|error| {
                error!("init admin server error: {error}");
                panic!("admin server error: {error}")
            }),
            init_http_server(config.clone(), service_config, domain_storage),
        );
    } else {
        tracing::info!("admin server disabled");
        init_http_server(config, service_config, domain_storage).await?
    }
    Ok(())
}

#[cfg(test)]
pub const LOCAL_HOST: &str = "local.fornetcode.com";
