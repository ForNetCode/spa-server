use crate::acme::ACMEManager;
use crate::config::SSL;
use crate::Config;
use anyhow::{anyhow, Context as _};
use core::task::{Context, Poll};
use dashmap::DashMap;
use futures_util::ready;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, AddrStream};
use rustls::pki_types::CertificateDer;
use rustls::server::{ClientHello, ResolvesServerCert, ResolvesServerCertUsingSni};
use rustls::sign::CertifiedKey;
use rustls::Error;
use std::fs::File;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::vec::Vec;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::rustls::ServerConfig;

//code from https://github.com/rustls/hyper-rustls/blob/main/examples/server.rs
// it's like warp/tls.rs

#[derive(Debug)]
struct AlwaysResolver(Arc<CertifiedKey>);
impl ResolvesServerCert for AlwaysResolver {
    fn resolve(&self, _: ClientHello) -> Option<Arc<CertifiedKey>> {
        Some(self.0.clone())
    }
}

#[derive(Debug)]
struct FileCertResolver {
    default: Option<Arc<CertifiedKey>>,
    wrapper: ResolvesServerCertUsingSni,
}

impl ResolvesServerCert for FileCertResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        self.wrapper.resolve(client_hello).or(self.default.clone())
    }
}

// code from ResolvesServerCertUsingSni
#[derive(Debug)]
struct DashMapCertResolver {
    dash_map: Arc<DashMap<String, Arc<CertifiedKey>>>,
}

impl ResolvesServerCert for DashMapCertResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        if let Some(name) = client_hello.server_name() {
            self.dash_map.get(name).map(|v| v.value().clone())
        } else {
            None
        }
    }
}

pub fn load_ssl_server_config(
    config: &Config,
    acme_manager: Arc<ACMEManager>,
) -> anyhow::Result<Option<Arc<ServerConfig>>> {
    let dynamic_resolver: Arc<dyn ResolvesServerCert> = if config
        .https
        .as_ref()
        .and_then(|v| v.acme.as_ref())
        .is_some()
    {
        Arc::new(DashMapCertResolver {
            dash_map: acme_manager.certificate_map.clone(),
        })
    } else if config.https.is_some() {
        let default = if let Some(ref ssl) = config.https.as_ref().and_then(|x| x.ssl.clone()) {
            Some(load_ssl_file(
                &PathBuf::from(&ssl.public),
                &PathBuf::from(&ssl.private),
            )?)
        } else {
            None
        };

        let ssls: Vec<(String, SSL)> = config
            .domains
            .iter()
            .filter_map(|domain| {
                domain
                    .https
                    .as_ref()
                    .map(|x| {
                        x.ssl
                            .as_ref()
                            .map(|ssl| (domain.domain.clone(), ssl.clone()))
                    })
                    .flatten()
            })
            .collect();

        if ssls.is_empty() && default.is_none() {
            return Err(anyhow!("no https certificate define"));
        } else if ssls.is_empty() {
            default
                .map(|cert_key| Arc::new(AlwaysResolver(Arc::new(cert_key))))
                .ok_or_else(|| anyhow!("The default https ssl can't parse correctly"))?
        } else {
            let mut wrapper = ResolvesServerCertUsingSni::new();
            for (domain, ssl) in ssls.iter() {
                let certified_key =
                    load_ssl_file(&PathBuf::from(&ssl.public), &PathBuf::from(&ssl.private))?;
                wrapper.add(domain, certified_key)?;
            }
            Arc::new(FileCertResolver {
                default: default.map(Arc::new),
                wrapper,
            })
        }
    } else {
        return Ok(None);
    };
    let mut cfg = ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(dynamic_resolver);
    cfg.alpn_protocols = vec!["h2".into(), "http/1.1".into()];
    Ok(Some(Arc::new(cfg)))
}

pub(crate) fn load_ssl_file(
    cert_path: &PathBuf,
    key_path: &PathBuf,
) -> anyhow::Result<CertifiedKey> {
    tracing::debug!("load cert:{:?}", cert_path);
    let cert_file =
        File::open(&cert_path).with_context(|| format!("fail to load cert:{:?}", cert_path))?;

    let mut reader = io::BufReader::new(cert_file);

    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<CertificateDer<'static>>, _>>()
        .with_context(|| format!("fail to parse cert:{:?}", cert_path))?;

    tracing::debug!("load key:{:?}", key_path);
    let key_file = File::open(key_path)
        .with_context(|| format!("fail to load private key:{:?}", cert_path))?;
    let mut reader = io::BufReader::new(key_file);

    let private_key = rustls_pemfile::private_key(&mut reader)?
        .ok_or_else(|| anyhow!("there's no private key"))?;
    let key = rustls::crypto::ring::sign::any_supported_type(&private_key)
        .map_err(|_| Error::General("invalid private key".into()))?;
    Ok(CertifiedKey::new(certs, key))
}

enum State {
    Handshaking(tokio_rustls::Accept<AddrStream>),
    Streaming(tokio_rustls::server::TlsStream<AddrStream>),
}

// tokio_rustls::server::TlsStream doesn't expose constructor methods,
// so we have to TlsAcceptor::accept and handshake to have access to it
// TlsStream implements AsyncRead/AsyncWrite handshaking tokio_rustls::Accept first
pub struct TlsStream {
    state: State,
}

impl TlsStream {
    fn new(stream: AddrStream, config: Arc<ServerConfig>) -> TlsStream {
        let accept = tokio_rustls::TlsAcceptor::from(config).accept(stream);
        TlsStream {
            state: State::Handshaking(accept),
        }
    }
}

impl AsyncRead for TlsStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_read(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TlsStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_write(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

pub struct TlsAcceptor {
    config: Arc<ServerConfig>,
    incoming: AddrIncoming,
}

impl TlsAcceptor {
    pub fn new(config: Arc<ServerConfig>, incoming: AddrIncoming) -> TlsAcceptor {
        TlsAcceptor { config, incoming }
    }
}

impl Accept for TlsAcceptor {
    type Conn = TlsStream;
    type Error = io::Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let pin = self.get_mut();
        match ready!(Pin::new(&mut pin.incoming).poll_accept(cx)) {
            Some(Ok(sock)) => Poll::Ready(Some(Ok(TlsStream::new(sock, pin.config.clone())))),
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}
