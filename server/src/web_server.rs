use crate::config::Config;
use crate::domain_storage::DomainStorage;
use crate::service::ServiceConfig;
use salvo::fs::NamedFile;
use salvo::http::uri::{Authority, PathAndQuery, Uri};
use salvo::http::{ParseError, ResBody};
use salvo::prelude::*;
use std::borrow::Cow;
use std::str::FromStr;
use std::sync::Arc;

#[inline]
pub(crate) fn decode_url_path_safely(path: &str) -> String {
    percent_encoding::percent_decode_str(path)
        .decode_utf8_lossy()
        .to_string()
}

#[inline]
pub(crate) fn format_url_path_safely(path: &str) -> String {
    let final_slash = if path.ends_with('/') { "/" } else { "" };
    let mut used_parts = Vec::with_capacity(8);
    for part in path.split(['/', '\\']) {
        if part.is_empty() || part == "." || (cfg!(windows) && part.contains(':')) {
            continue;
        }
        if part == ".." {
            used_parts.pop();
        } else {
            used_parts.push(part);
        }
    }
    used_parts.join("/") + final_slash
}

fn get_authority(req: &Request) -> Option<Authority> {
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

fn replace_uri_path(original_uri: &Uri, new_path: &str) -> Result<Uri, ParseError> {
    let mut uri_parts = original_uri.clone().into_parts();
    uri_parts.authority = None;
    uri_parts.scheme = None;
    let path = match original_uri.query() {
        Some(query) => Cow::from(format!("{new_path}?{query}")),
        None => Cow::from(new_path),
    };

    uri_parts.path_and_query = Some(PathAndQuery::from_str(path.as_ref())?);
    Ok(Uri::from_parts(uri_parts)?)
}

#[handler]
async fn file_resp(req: &mut Request, depot: &mut Depot, res: &mut Response, _ctrl: &mut FlowCtrl) {
    let domain_storage = depot.obtain::<Arc<DomainStorage>>().unwrap();
    let service_config = depot.obtain::<Arc<ServiceConfig>>().unwrap();
    // let config = depot.obtain::<Arc<Config>>().unwrap();

    let author_opt = get_authority(req);
    if let Some(author) = author_opt {
        let origin_host = author.host();

        let host = if let Some(alias) = service_config.host_alias.get(origin_host) {
            alias.as_str()
        } else {
            origin_host
        };

        let uri = req.uri();

        let req_path = uri.path();

        let rel_path = if let Some(rest) = req.params().tail() {
            rest
        } else {
            &*decode_url_path_safely(req_path)
        };
        let rel_path = format_url_path_safely(rel_path);
        // tracing::debug!("hit {rel_path}");
        match domain_storage.get_file(host, &rel_path) {
            Some(item) => {
                NamedFile::builder(&item.data)
                    .send(req.headers(), res)
                    .await;
            }
            None => {
                // trailing slash
                let original_path = req.uri().path();
                if !original_path.is_empty() && original_path != "/" {
                    let ends_with_slash = original_path.ends_with('/');

                    if !ends_with_slash
                        && let Ok(new_uri) =
                            replace_uri_path(req.uri(), &format!("{original_path}/"))
                    {
                        res.body(ResBody::None);

                        match Redirect::with_status_code(StatusCode::MOVED_PERMANENTLY, new_uri) {
                            Ok(redirect) => {
                                res.render(redirect);
                            }
                            Err(e) => {
                                tracing::error!(error = ?e, "redirect failed");
                            }
                        }
                        return;
                    }
                }
                res.status_code(StatusCode::NOT_FOUND);
            }
        }
    } else {
        res.status_code(StatusCode::FORBIDDEN);
    }
}

pub async fn init_http_server(
    conf: Arc<Config>,
    service_config: Arc<ServiceConfig>,
    storage: Arc<DomainStorage>,
) -> anyhow::Result<()> {
    let http_config = &conf.http;
    let listener = TcpListener::new((http_config.addr.clone(), http_config.port))
        .bind()
        .await;

    // StaticDir::new();
    let router = Router::with_hoop(
        affix_state::inject(service_config.clone())
            .inject(storage.clone())
            .inject(conf.clone()),
    )
    .path("{*path}")
    .get(file_resp);

    Server::new(listener).serve(router).await;
    Ok(())
}
