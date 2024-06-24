use crate::acme::{get_challenge_path, ChallengePath, ACME_CHALLENGE};
use crate::cors::{cors_resp, resp_cors_request, Validated};
use crate::DomainStorage;
use futures_util::future::Either;
use headers::HeaderMapExt;
use hyper::header::LOCATION;
use hyper::http::uri::Authority;
use hyper::{Body, Request, Response, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use tracing::warn;
use warp::fs::{ArcPath, Conditionals};
use warp::Reply;

use crate::static_file_filter::{cache_or_file_reply, get_cache_file};

pub struct ServiceConfig {
    pub default: DomainServiceConfig,
    pub inner: HashMap<String, DomainServiceConfig>,
}

pub struct DomainServiceConfig {
    pub cors: bool,
    pub http_redirect_to_https: Option<u32>,
    pub enable_acme: bool,
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
    challenge_path: ChallengePath,
    is_https: bool,
) -> Result<warp::reply::Response, Infallible> {
    let uri = req.uri();
    let from_uri = uri.authority().cloned();
    // trick, need more check
    let authority_opt = from_uri.or_else(|| {
        req.headers().get("host").and_then(|value| {
            value
                .to_str()
                .ok()
                .and_then(|x| Authority::from_str(x).ok())
        })
    });

    if let Some(authority) = authority_opt {
        let host = authority.host();

        let service_config = service_config.get_domain_service_config(host);
        // cors
        let origin_opt = match resp_cors_request(req.method(), req.headers(), service_config.cors) {
            Either::Left(x) => Some(x),
            Either::Right(v) => return Ok(v),
        };
        let path = uri.path();
        // redirect to https
        if !is_https {
            // check if acme challenge
            if service_config.enable_acme && path.starts_with(ACME_CHALLENGE) {
                let token = &path[ACME_CHALLENGE.len()..];
                {
                    if let Some(path) = challenge_path.read().await.as_ref() {
                        let path = get_challenge_path(path, host, token);
                        let headers = req.headers();
                        let conditionals = Conditionals {
                            if_modified_since: headers.typed_get(),
                            if_unmodified_since: headers.typed_get(),
                            if_range: headers.typed_get(),
                            range: headers.typed_get(),
                        };
                        return match warp::fs::file_reply(ArcPath(Arc::new(path)), conditionals)
                            .await
                        {
                            Ok(resp) => Ok(resp.into_response()),
                            Err(_err) => {
                                warn!("known challenge error:{_err:?}");
                                Ok(not_found())
                            }
                        };
                    }
                }
                return Ok(not_found());
            }
            if let Some(port) = service_config.http_redirect_to_https {
                let mut resp = Response::default();
                let redirect_path = if port != 443 {
                    format!("https://{host}:{port}{uri}")
                } else {
                    format!("https://{host}{uri}")
                };
                resp.headers_mut()
                    .insert(LOCATION, redirect_path.parse().unwrap());
                *resp.status_mut() = StatusCode::MOVED_PERMANENTLY;
                return Ok(resp);
            }
        }
        // path: "" => "/"
        if domain_storage.check_if_empty_index(host, path) {
            let mut resp = Response::default();
            let mut path = format!("{path}/");
            if let Some(query) = uri.query() {
                path.push('?');
                path.push_str(query);
            }
            let path = path.parse().unwrap();
            resp.headers_mut().insert(LOCATION, path);
            *resp.status_mut() = StatusCode::MOVED_PERMANENTLY;
            return Ok(resp);
        }
        // static file
        let mut resp = match get_cache_file(path, host, domain_storage).await {
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
                    .and_then(|x| x.to_str().map(|x| x.to_string()).ok());
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

// get version
/*
let uri = req.uri();
let path = uri.path();
if path.ends_with("/_api") {
    //TODO: add switch: if exposed _api to public.
    if let Some((domain_with_sub, _)) = path.rsplit_once("/_version/_api") {
        let path = if domain_with_sub == "" {
            host.to_string()
        } else {
            format!("{host}/{domain_with_sub}")
        };
        let version = domain_storage
            .get_domain_info_by_domain(&path)
            .map(|info| info.current_version)
            .flatten()
            .unwrap_or(0)
            .to_string();
        let resp = Body::from(version);
        let resp = Response::new(resp);
        return Ok(resp);
    } else if let Some((domain_with_sub, _)) = path.rsplit_once("/_files/_api") {
        let path = if domain_with_sub == "" {
            host.to_string()
        } else {
            format!("{host}/{domain_with_sub}")
        };
        let version = domain_storage
            .get_domain_info_by_domain(&path)
            .map(|info| info.current_version)
            .flatten()
            .unwrap_or(0);

        return match domain_storage.get_files_metadata(path, version) {
            Ok(data) => Ok(warp::reply::json(&data).into_response()),
            Err(e) => {
                let mut resp = Response::new(Body::from(format!("error:{}", e)));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                Ok(resp)
            }
        };
    }
    let mut resp = Response::default();
    *resp.status_mut() = StatusCode::NOT_FOUND;
    return Ok(resp);
}
 */
