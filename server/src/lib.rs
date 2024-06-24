//#![allow(dead_code)]
//#![allow(unused_variables)]
pub mod web_server;

pub mod admin_server;
pub mod config;
pub mod domain_storage;
pub mod file_cache;
pub mod hot_reload;
pub mod tls;

mod acme;
pub mod cors;
pub mod service;
pub mod static_file_filter;

use crate::acme::{ACMEManager, RefreshDomainMessage, ReloadACMEState};
use crate::admin_server::AdminServer;
use crate::config::{AdminConfig, Config};
use crate::domain_storage::DomainStorage;
use crate::file_cache::FileCache;
use crate::hot_reload::{HotReloadManager, OneShotReloadState};
use crate::tls::load_ssl_server_config;
use anyhow::bail;
use delay_timer::entity::DelayTimer;
use delay_timer::prelude::DelayTimerBuilder;
use futures::future::join;
use futures_util::TryFutureExt;
use if_chain::if_chain;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::error;
use warp::Filter;
pub use web_server::Server;

pub fn with<T: Send + Sync>(
    d: Arc<T>,
) -> impl Filter<Extract = (Arc<T>,), Error = Infallible> + Clone {
    warp::any().map(move || d.clone())
}

async fn run_admin_server(
    config: &AdminConfig,
    storage: &Arc<DomainStorage>,
    reload_manager: HotReloadManager,
    acme_manager: Arc<ACMEManager>,
    delay_timer: DelayTimer,
) -> anyhow::Result<()> {
    let admin_server = AdminServer::new(
        config,
        storage.clone(),
        reload_manager,
        acme_manager,
        delay_timer,
    );
    admin_server.run().await
}

pub async fn reload_server(
    admin_config: &AdminConfig,
    reload_manager: &HotReloadManager,
    acme_manager: Arc<ACMEManager>,
) -> anyhow::Result<()> {
    // TODO:
    // check: if port can bind.
    let config = Config::load()?;
    if config.admin_config.as_ref() == Some(admin_config) {
        let cache = FileCache::new(&config);
        let domain_storage = Arc::new(DomainStorage::init(&config.file_dir, cache)?);

        let (state, http_rx, https_rx) = OneShotReloadState::init(&config);
        let server = Server::new(config.clone(), domain_storage.clone());
        let acme_config = config.https.as_ref().and_then(|x| x.acme.clone());
        let reload_acme_state: Option<ReloadACMEState> = if let Some(acme_config) = acme_config {
            Some(ACMEManager::init_acme_provider_and_certificate(
                &config,
                acme_config,
                domain_storage,
                acme_manager.certificate_map.clone(),
            )?)
        } else {
            None
        };
        let challenge_path = reload_acme_state
            .as_ref()
            .map(|r| r.challenge_path.as_ref().clone());
        {
            let mut path = acme_manager.challenge_dir.write().await;
            *path = challenge_path;
        }
        let challenge_path = acme_manager.challenge_dir.clone();
        let _sender = acme_manager.sender.clone();
        let tls_server_config = load_ssl_server_config(&config, acme_manager)?;
        tokio::task::spawn(async move {
            join(
                server
                    .init_http_server(http_rx, challenge_path.clone())
                    .map_err(|error| error!("reload http server error:{error}")),
                server
                    .init_https_server(https_rx, tls_server_config, challenge_path.clone())
                    .map_err(|error| error!("reload https server error:{error}")),
            )
            .await
        });
        // sleep 500
        tokio::time::sleep(Duration::from_millis(500)).await;
        reload_manager.reload(state, reload_acme_state).await?;

        tokio::spawn(async move {
            sleep(Duration::from_secs(3)).await;
            let _ = _sender.send(RefreshDomainMessage(vec![])).await;
        });
    }
    Ok(())
}

pub async fn run_server() -> anyhow::Result<()> {
    let config = Config::load()?;
    tracing::debug!("config load:{:?}", &config);
    run_server_with_config(config).await
}

pub async fn run_server_with_config(config: Config) -> anyhow::Result<()> {
    let cache = FileCache::new(&config);
    let domain_storage = Arc::new(DomainStorage::init(&config.file_dir, cache)?);
    let server = Server::new(config.clone(), domain_storage.clone());

    if let Some(admin_config) = &config.admin_config {
        tracing::info!("admin server enabled");
        if_chain! {
            if let Some(http_config) = &config.https;
            if let Some(_) = &http_config.acme;
            then {
                if config.domains.iter().any(|x|x.https.as_ref().map(|x|x.ssl.is_some()).unwrap_or(false)) ||
                http_config.ssl.is_some() {
                    let msg = "https certificate file and acme don't support together";
                    error!(msg);
                    bail!(msg)
                }
            }
        }
        let (reload_manager, http_rx, https_rx, acme_rx) = HotReloadManager::init(&config);
        let delay_timer = DelayTimerBuilder::default()
            .tokio_runtime_by_default()
            .build();

        let acme_manager = Arc::new(ACMEManager::init(
            &config,
            domain_storage.clone(),
            Some(acme_rx),
            &delay_timer,
        )?);
        let challenge_path = acme_manager.challenge_dir.clone();

        let tls_server_config = load_ssl_server_config(&config, acme_manager.clone())?;
        let _ = tokio::join!(
            server
                .init_https_server(https_rx, tls_server_config, challenge_path.clone())
                .map_err(|error| {
                    error!("init https server error: {error}");
                    error
                }),
            server
                .init_http_server(http_rx, challenge_path.clone())
                .map_err(|error| {
                    error!("init http server error: {error}");
                    error
                }),
            run_admin_server(
                admin_config,
                &domain_storage,
                reload_manager,
                acme_manager.clone(),
                delay_timer,
            )
            .map_err(|error| {
                error!("init admin server error: {error}");
                panic!("admin server error: {error}")
            })
        );
    } else {
        tracing::info!("admin server disabled");

        let delay_timer = DelayTimerBuilder::default()
            .tokio_runtime_by_default()
            .build();

        let acme_manager = Arc::new(ACMEManager::init(
            &config,
            domain_storage.clone(),
            None,
            &delay_timer,
        )?);
        let challenge_path = acme_manager.challenge_dir.clone();
        let tls_server_config = load_ssl_server_config(&config, acme_manager.clone())?;
        let _ = tokio::join!(
            server
                .init_https_server(None, tls_server_config, challenge_path.clone())
                .map_err(|error| {
                    error!("init https server error: {error}");
                    panic!("init https server error: {error}")
                }),
            server
                .init_http_server(None, challenge_path)
                .map_err(|error| {
                    error!("init http server error: {error}");
                    panic!("init http server error: {error}")
                }),
        );
    }
    Ok(())
}

#[cfg(test)]
pub const LOCAL_HOST: &str = "local.fornetcode.com";
