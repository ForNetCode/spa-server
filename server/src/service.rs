use crate::cors::{cors_resp, resp_cors_request, Validated};
use crate::DomainStorage;
use futures_util::future::Either;
use headers::HeaderMapExt;
use hyper::header::LOCATION;
use hyper::http::uri::{Authority, Scheme};
use hyper::{Body, Request, Response, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use warp::fs::Conditionals;

use crate::static_file_filter::{cache_or_file_reply, get_cache_file};

pub struct ServiceConfig {
    pub default: DomainServiceConfig,
    pub inner: HashMap<String, DomainServiceConfig>,
}

pub struct DomainServiceConfig {
    pub cors: bool,
    pub http_redirect_to_https: bool,
}

impl ServiceConfig {
    fn get_domain_service_config(&self, domain: &str) -> &DomainServiceConfig {
        self.inner.get(domain).unwrap_or(&self.default)
    }
}

pub async fn create_service(
    req: Request<Body>,
    service_config: Arc<ServiceConfig>,
    domain_storage: Arc<DomainStorage>,
) -> Result<Response<Body>, Infallible> {
    let from_uri = req.uri().authority().cloned();

    // trick, need more check
    let authority_opt = from_uri.or_else(|| {
        req.headers()
            .get("host")
            .map(|value| {
                value
                    .to_str()
                    .ok()
                    .map(|x| Authority::from_str(x).ok())
                    .flatten()
            })
            .flatten()
    });

    if let Some(authority) = authority_opt {
        let host = authority.host();
        let service_config = service_config.get_domain_service_config(host);
        // cors
        let origin_opt = match resp_cors_request(req.method(), req.headers(), service_config.cors) {
            Either::Left(x) => Some(x),
            Either::Right(v) => return Ok(v),
        };

        let scheme = req.uri().scheme();
        // redirect to https
        if scheme == Some(&Scheme::HTTP) && service_config.http_redirect_to_https {
            let mut resp = Response::default();
            resp.headers_mut()
                .insert(LOCATION, req.uri().to_string().parse().unwrap());
            *resp.status_mut() = StatusCode::MOVED_PERMANENTLY;
            return Ok(resp);
        }
        // get version

        if req.uri().path() == "/_version" {
            let version = domain_storage
                .get_domain_info_by_domain(host)
                .map(|info| info.current_version)
                .flatten()
                .unwrap_or(0)
                .to_string();
            let resp = Body::from(version);
            let resp = Response::new(resp);
            return Ok(resp);
        }
        // static file
        let mut resp = match get_cache_file(req.uri().path(), host, domain_storage).await {
            Ok(item) => {
                let headers = req.headers();
                let conditionals = Conditionals {
                    if_modified_since: headers.typed_get(),
                    if_unmodified_since: headers.typed_get(),
                    if_range: headers.typed_get(),
                    range: headers.typed_get(),
                };
                let accept_encoding = headers
                    .get("accept-encoding")
                    .map(|x| x.to_str().map(|x| x.to_string()).ok())
                    .flatten();
                cache_or_file_reply(item, conditionals, accept_encoding).await
            }
            Err(resp) => Ok(resp),
        };

        if let Some(Validated::Simple(origin)) = origin_opt {
            resp = resp.map(|r| cors_resp(r, origin));
        }
        resp
    } else {
        let mut resp = Response::default();
        *resp.status_mut() = StatusCode::FORBIDDEN;
        Ok(resp)
    }
}

pub fn not_found() -> Response<Body> {
    let mut resp = Response::default();
    *resp.status_mut() = StatusCode::NOT_FOUND;
    resp
}

pub fn resp(code: StatusCode, str: &'static str) -> Response<Body> {
    let mut resp = Response::new(Body::from(str));
    *resp.status_mut() = code;
    resp
}
