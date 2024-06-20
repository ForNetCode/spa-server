use chrono::{DateTime, Local};
use hyper::server::conn::AddrIncoming;
use hyper::server::Server as HServer;
use hyper::service::service_fn;
use socket2::{Domain, Socket, Type};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{SocketAddr, TcpListener};
use std::str::FromStr;
use std::sync::Arc;
use rustls::ServerConfig;
use tokio::net::TcpListener as TKTcpListener;
use tokio::sync::oneshot::Receiver;

use crate::config::Config;
use crate::domain_storage::DomainStorage;
use crate::service::{create_service, DomainServiceConfig, ServiceConfig};
use crate::tls::TlsAcceptor;

async fn handler(rx: Receiver<()>, time: DateTime<Local>, http_or_https: &'static str) {
    rx.await.ok();
    tracing::info!(
        "prepare to close {} server which start at {}",
        http_or_https,
        time.format("%Y-%m-%d %H:%M:%S"),
    );
}
macro_rules! run_server {
    ($server:ident, $rx:ident) => {
        let time = Local::now();
        if $rx.is_some() {
            let h = handler($rx.unwrap(), time, "http");
            $server.with_graceful_shutdown(h).await?;
        } else {
            $server.await?;
        }
    };

    (tls: $server:ident, $rx:ident) => {
        let time = Local::now();
        if $rx.is_some() {
            let h = handler($rx.unwrap(), time, "https");
            $server.with_graceful_shutdown(h).await?;
        } else {
            $server.await?;
        };
    };
}

pub struct Server {
    conf: Config,
    storage: Arc<DomainStorage>,
    service_config: Arc<ServiceConfig>,
}

impl Server {
    pub fn new(conf: Config, storage: Arc<DomainStorage>) -> Self {
        let default_http_redirect_to_https = conf
            .https
            .as_ref()
            .map(|x| x.http_redirect_to_https);
        let default = DomainServiceConfig {
            cors: conf.cors,
            http_redirect_to_https: default_http_redirect_to_https,
        };
        let service_config: HashMap<String, DomainServiceConfig> = conf
            .domains
            .iter()
            .map(|domain| {
                let domain_service_config: DomainServiceConfig = DomainServiceConfig {
                    cors: domain.cors.unwrap_or(default.cors),
                    http_redirect_to_https: domain
                        .https
                        .as_ref()
                        .and_then(|x| x.http_redirect_to_https)
                        .or(default_http_redirect_to_https),
                };

                (domain.domain.clone(), domain_service_config)
            })
            .collect();

        let service_config = Arc::new(ServiceConfig {
            default,
            inner: service_config,
        });
        Server {
            conf,
            storage,
            service_config,
        }
    }

    pub async fn init_https_server(
        &self,
        rx: Option<Receiver<()>>,
        tls_server_config: Option<Arc<ServerConfig>>,
    ) -> anyhow::Result<()> {
        if let Some(config) = &self.conf.https {
            // This has checked by load_ssl_server_config
            let tls_server_config = tls_server_config.unwrap();
            let bind_address =
                SocketAddr::from_str(&format!("{}:{}", &config.addr, &config.port)).unwrap();

            let make_svc = hyper::service::make_service_fn(|_| {
                let service_config = self.service_config.clone();
                let storage = self.storage.clone();
                async move {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        create_service(req, service_config.clone(), storage.clone(), true)
                    }))
                }
            });
            tracing::info!("listening on https://{}", &bind_address);
            let incoming =
                AddrIncoming::from_listener(TKTcpListener::from_std(get_socket(bind_address)?)?)?;

            let server =
                HServer::builder(TlsAcceptor::new(tls_server_config, incoming)).serve(make_svc);
            run_server!(tls: server, rx);
        }
        Ok(())
    }

    pub async fn init_http_server(&self, rx: Option<Receiver<()>>) -> anyhow::Result<()> {
        if let Some(http_config) = &self.conf.http {
            let bind_address =
                SocketAddr::from_str(&format!("{}:{}", &http_config.addr, &http_config.port)).unwrap();
            let make_svc = hyper::service::make_service_fn(|_| {
                let service_config = self.service_config.clone();
                let storage = self.storage.clone();

                async move {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        create_service(req, service_config.clone(), storage.clone(), false)
                    }))
                }
            });
            tracing::info!("listening on http://{}", &bind_address);
            let server = HServer::from_tcp(get_socket(bind_address)?)?.serve(make_svc);
            run_server!(server, rx);
        }
        Ok(())
    }
}

pub fn get_socket(address: SocketAddr) -> anyhow::Result<TcpListener> {
    let socket = Socket::new(Domain::for_address(address), Type::STREAM, None)?;
    socket.set_nodelay(true)?;
    // socket.set_reuse_address(true)?;
    #[cfg(not(any(target_os = "solaris", target_os = "illumos", target_os = "windows")))]
    socket.set_reuse_port(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&address.into())?;
    socket.listen(128)?;
    let listener: TcpListener = socket.into();
    Ok(listener)
}
