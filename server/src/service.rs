use crate::config::{Config, extract_origin};
use salvo::Response;
use salvo::http::{HeaderValue, StatusCode};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct ServiceConfig {
    pub default: DomainServiceConfig,
    pub inner: HashMap<String, DomainServiceConfig>,
    pub host_alias: Arc<HashMap<String, String>>,
}

pub struct DomainServiceConfig {
    pub cors: Option<HashSet<HeaderValue>>,
}

impl ServiceConfig {
    /*
    fn get_domain_service_config(&self, domain: &str) -> &DomainServiceConfig {
        self.inner.get(domain).unwrap_or(&self.default)
    }

     */
    pub fn new(conf: &Config) -> Self {
        let default = DomainServiceConfig {
            cors: extract_origin(&conf.cors),
        };
        let mut service_config: HashMap<String, DomainServiceConfig> = HashMap::new();
        for domain in conf.domains.iter() {
            let domain_service_config: DomainServiceConfig = DomainServiceConfig {
                cors: extract_origin(&domain.cors).or_else(|| default.cors.clone()),
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

        ServiceConfig {
            default,
            inner: service_config,
            host_alias: Arc::new(alias_map),
        }
    }
}

// static file reply
/*
#[instrument(skip(uri, host, domain_storage, origin_opt))]
async fn file_resp(
    req: &warp::Request,
    uri: &Uri,
    host: &str,
    domain_storage: Arc<DomainStorage>,
    origin_opt: Option<Validated>,
) -> Result<Response<Body>, Infallible> {
    let path = uri.path();
    let mut resp = match get_cache_file(path, host, domain_storage.clone()).await {
        Some(item) => {
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
        None => {
            // path: "" => "/"
            if domain_storage.check_if_empty_index(host, path) {
                let mut resp = Response::default();
                //Attention: alias would be twice
                let mut path = format!("{path}/");
                if let Some(query) = uri.query() {
                    path.push('?');
                    path.push_str(query);
                }
                let path = path.parse().unwrap();
                resp.headers_mut().insert(LOCATION, path);
                *resp.status_mut() = StatusCode::MOVED_PERMANENTLY;
                Ok(resp)
            } else {
                Ok(not_found())
            }
        }
    };
    if let Some(Validated::Simple(origin)) = origin_opt {
        resp = resp.map(|r| cors_resp(r, origin));
    }
    resp
}

 */
/*
fn get_authority(req: &Request<Body>) -> Option<Authority> {
    let uri = req.uri();
    let from_uri = uri.authority().cloned();
    // trick, need more check
    from_uri.or_else(|| {
        req.headers().get("host").and_then(|value| {
            value
                .to_str()
                .ok()
                .and_then(|x| Authority::from_str(x).ok())
        })
    })
}

*/

/*
pub async fn create_https_service(
    req: Request<Body>,
    service_config: Arc<ServiceConfig>,
    domain_storage: Arc<DomainStorage>,
    external_port: u16,
) -> Result<warp::reply::Response, Infallible> {
    let authority_opt = get_authority(&req);

    if let Some(authority) = authority_opt {
        let origin_host = authority.host();
        let (is_alias, host) = if let Some(alias) = service_config.host_alias.get(origin_host) {
            (true, alias.as_str())
        } else {
            (false, origin_host)
        };

        let service_config = service_config.get_domain_service_config(host);
        // cors
        let origin_opt = match resp_cors_request(req.method(), req.headers(), &service_config.cors)
        {
            Either::Left(x) => Some(x),
            Either::Right(v) => return Ok(v),
        };
        let uri = req.uri();
        if is_alias {
            return Ok(alias_redirect(uri, true, host, external_port));
        }
        file_resp(&req, uri, host, domain_storage, origin_opt).await
    } else {
        Ok(forbid())
    }
}

 */

pub fn not_found() -> Response {
    let mut res = Response::new();
    res.status_code(StatusCode::NOT_FOUND);
    res
}
pub fn forbid() -> Response {
    let mut res = Response::new();
    res.status_code(StatusCode::FORBIDDEN);
    res
}
