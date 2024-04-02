use crate::config::SSL;
use crate::Config;
use anyhow::{anyhow, Context as _};
use core::task::{Context, Poll};
use futures_util::ready;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, AddrStream};
use rustls::server::{ClientHello, ResolvesServerCert, ResolvesServerCertUsingSni};
use rustls::sign::CertifiedKey;
use rustls::{sign, Error};
use std::fs::File;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::vec::Vec;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer};
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
struct CertResolver {
    default: Option<Arc<CertifiedKey>>,
    wrapper: ResolvesServerCertUsingSni,
}

pub fn load_ssl_server_config(config: &Config) -> anyhow::Result<Arc<ServerConfig>> {
    let default = if let Some(ref ssl) = config.https.as_ref().map(|x| x.ssl.clone()).flatten() {
        Some(load_ssl_file(ssl)?)
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

    let dynamic_resolver: Arc<dyn ResolvesServerCert> = if ssls.is_empty() && default.is_none() {
        return Err(anyhow!(""));
    } else if ssls.is_empty() {
        default
            .map(|cert_key| Arc::new(AlwaysResolver(Arc::new(cert_key))))
            .ok_or_else(|| anyhow!("The default https ssl can't parse correctly"))?
    } else {
        let mut wrapper = ResolvesServerCertUsingSni::new();
        for (domain, ssl) in ssls.iter() {
            let certified_key = load_ssl_file(ssl)?;
            wrapper.add(domain, certified_key)?;
        }
        Arc::new(CertResolver {
            default: default.map(Arc::new),
            wrapper,
        })
    };
    let mut cfg = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(dynamic_resolver);
    cfg.alpn_protocols = vec!["h2".into(), "http/1.1".into()];
    Ok(Arc::new(cfg))
}

impl ResolvesServerCert for CertResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        self.wrapper.resolve(client_hello).or(self.default.clone())
    }
}

fn load_ssl_file(ssl: &SSL) -> anyhow::Result<sign::CertifiedKey> {
    let cert_path = &ssl.public;
    let key_path = &ssl.private;

    tracing::debug!("load cert:{}", cert_path);
    let cert_file =
        File::open(cert_path).with_context(|| format!("fail to load cert:{}", cert_path))?;

    let mut reader = io::BufReader::new(cert_file);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<CertificateDer<'static>>, _>>()
        .with_context(|| format!("fail to parse cert:{}", cert_path))?;


    tracing::debug!("load key:{}", key_path);
    let key_file =
        File::open(key_path).with_context(|| format!("fail to load private key:{}", cert_path))?;
    let mut reader = io::BufReader::new(key_file);
    let keys = rustls_pemfile::rsa_private_keys(&mut reader)
        .collect::<Result<Vec<PrivatePkcs1KeyDer<'static>>, _>>()
        .with_context(|| format!("fail to parse private key:{}", cert_path))?;
    if keys.len() != 1 {
        return Err(anyhow!("expected a single private key"));
    }

    let private_key = PrivateKeyDer::from(keys.into_iter().nth(0).unwrap());
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
