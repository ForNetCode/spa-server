use crate::acme::ChallengePath;
use chrono::{DateTime, Local};
use hyper::server::conn::AddrIncoming;
use hyper::server::{Server as HServer};
use hyper::service::service_fn;
use rustls::ServerConfig;
use socket2::{Domain, Socket, Type};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{SocketAddr, TcpListener};
use std::str::FromStr;
use std::sync::Arc;
use anyhow::bail;
use futures_util::future::Either;
use tokio::net::TcpListener as TKTcpListener;
use tokio::sync::oneshot::Receiver;

use crate::config::{Config, HttpConfig, HttpsConfig};
use crate::domain_storage::DomainStorage;
use crate::service::{create_http_service, create_https_service, DomainServiceConfig, ServiceConfig};
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
    pub fn new(conf: Config, storage: Arc<DomainStorage>) -> anyhow::Result<Self> {
        let default_http_redirect_to_https:Option<Either<&'static str, u16>> = conf.http.as_ref().and_then(|x| {
            match x.redirect_https {
                Some(true) => {
                    let external_port = conf.https.as_ref().and_then(|https| https.external_port);
                    if external_port.is_none() {
                        Some(Either::Left("when redirect_https is undefined or true, https.external_port should be set"))
                    } else {
                        external_port.map(|x|Either::Right(x))
                    }
                },
                None => {
                    match &conf.https {
                        Some(https) => {
                            let external_port = https.external_port;
                            if external_port.is_none() {
                                Some(Either::Left("when redirect_https is undefined or true, https.external_port should be set"))
                            } else {
                                external_port.map(|x|Either::Right(x))
                            }
                        },
                        None => None,
                    }
                },
                Some(false) => {
                    None
                }
            }
        });
        let default_http_redirect_to_https = match default_http_redirect_to_https {
            Some(Either::Right(v)) => Some(v),
            None => None,
            Some(Either::Left(s)) => bail!(s)
        };

        let default = DomainServiceConfig {
            cors: conf.cors,
            redirect_https: default_http_redirect_to_https,
            enable_acme: conf.https.as_ref().and_then(|x| x.acme.as_ref()).is_some(),
        };
        let mut service_config: HashMap<String, DomainServiceConfig> = HashMap::new();
        for domain in conf.domains.iter() {
            let redirect_https = match domain.redirect_https {
                None => default_http_redirect_to_https,
                Some(true) =>  {
                    match default_http_redirect_to_https {
                        Some(port) => Some(port),
                        None => {
                            let external_port = conf.https.as_ref().and_then(|https| https.external_port);
                            if external_port.is_none() {
                                bail!("when domains.redirect_https is true, https.external_port should be set")
                            }
                            external_port
                        }
                    }
                },
                Some(false) => None
            };
            let domain_service_config: DomainServiceConfig = DomainServiceConfig {
                cors: domain.cors.unwrap_or(default.cors),
                redirect_https,
                enable_acme: domain
                    .https
                    .as_ref()
                    .map(|x| x.disable_acme)
                    .unwrap_or(default.enable_acme),
            };
            service_config.insert(domain.domain.clone(), domain_service_config);
        }

        let mut alias_map = HashMap::new();
        for domain in conf.domains.iter() {
            if let Some(alias_host_list) = domain.alias.as_ref() {
                for alias_host in alias_host_list {
                    alias_map.insert(alias_host.clone(), domain.domain.clone());
                }
            }
        }

        let service_config = Arc::new(ServiceConfig {
            default,
            inner: service_config,
            host_alias: Arc::new(alias_map),
        });
        Ok(Server {
            conf,
            storage,
            service_config,
        })
    }
    pub fn init_http_tcp(http_config: &HttpConfig) -> anyhow::Result<TcpListener> {
        let bind_address =
            SocketAddr::from_str(&format!("{}:{}", &http_config.addr, &http_config.port))?;
        let socket= get_socket(bind_address)?;
        Ok(socket)
    }

    fn init_https_tcp(config:&HttpsConfig, tls_server_config:Arc<ServerConfig>) -> anyhow::Result<(SocketAddr, TlsAcceptor)> {
        let bind_address =
            SocketAddr::from_str(&format!("{}:{}", &config.addr, &config.port))?;

        let incoming =
            AddrIncoming::from_listener(TKTcpListener::from_std(get_socket(bind_address)?)?)?;
        let local_addr = incoming.local_addr();
        Ok((local_addr, TlsAcceptor::new(tls_server_config, incoming)))
    }

    pub async fn init_https_server(
        &self,
        rx: Option<Receiver<()>>,
        tls_server_config: Option<Arc<ServerConfig>>,
    ) -> anyhow::Result<()> {
        if let Some(config) = &self.conf.https {
            // This has checked by load_ssl_server_config

            let tls_server_config = tls_server_config.unwrap();
            let (local_addr, acceptor ) = Self::init_https_tcp(config, tls_server_config)?;
            tracing::info!("listening on https://{}", local_addr);
            let external_port = config.external_port.unwrap_or(local_addr.port());

            let make_svc = hyper::service::make_service_fn(|_| {
                let service_config = self.service_config.clone();
                let storage = self.storage.clone();
                async move {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        create_https_service(req, service_config.clone(), storage.clone(), external_port)
                    }))
                }
            });

            let server =
                HServer::builder(acceptor).serve(make_svc);
            run_server!(tls: server, rx);
        }
        Ok(())
    }

    pub async fn init_http_server(
        &self,
        rx: Option<Receiver<()>>,
        challenge_path: ChallengePath,
    ) -> anyhow::Result<()> {
        if let Some(http_config) = &self.conf.http {
            let listener = Self::init_http_tcp(http_config)?;
            let local_addr = listener.local_addr()?;
            tracing::info!("listening on http://{}", &local_addr);
            let external_port = http_config.external_port.unwrap_or(local_addr.port());
            let make_svc = hyper::service::make_service_fn(|_| {
                let service_config = self.service_config.clone();
                let storage = self.storage.clone();
                let challenge_path = challenge_path.clone();
                async move {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        create_http_service(req, service_config.clone(), storage.clone(), challenge_path.clone(), external_port)

                    }))
                }
            });
            let server = HServer::from_tcp(listener)?.serve(make_svc);
            run_server!(server, rx);
        }
        Ok(())
    }
    pub fn get_host_alias(&self) -> Arc<HashMap<String, String>> {
        self.service_config.host_alias.clone()
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
