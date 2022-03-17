use crate::domain_storage::DomainStorage;
use crate::file_cache::CacheItem;
use crate::with;
use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt, LastModified};
use hyper::body::Bytes;
use hyper::Body;
use percent_encoding::percent_decode_str;
use std::sync::Arc;
use warp::fs::{conditionals, Cond, Conditionals};
use warp::host::Authority;
use warp::http::Response;
use warp::{reject, Filter, Rejection};

fn cache_reply(item: Arc<CacheItem>, conditionals: Conditionals) -> Response<Body> {
    //let CacheItem { data, mime, meta } = ;
    let modified = item.meta.modified().map(LastModified::from).ok();
    let len = item.meta.len();

    let resp = match conditionals.check(modified) {
        Cond::NoBody(resp) => resp,
        Cond::WithBody(range) => {
            let data2 = Body::from(Bytes::new());
            let mut resp = Response::new(Body::from(item.data.clone()));
            resp.headers_mut().typed_insert(ContentLength(len));
            resp.headers_mut()
                .typed_insert(ContentType::from(item.mime.clone()));
            resp.headers_mut().typed_insert(AcceptRanges::bytes());
            if let Some(last_modified) = modified {
                resp.headers_mut().typed_insert(last_modified);
            }
            resp
        }
    };
    resp
}
pub fn static_file_filter(
    domain_storage: Arc<DomainStorage>,
) -> impl Filter<Extract = (Response<Body>,), Error = Rejection> + Clone {
    async fn get_cache_file(
        tail: warp::path::Tail,
        authority_opt: Option<Authority>,
        domain_storage: Arc<DomainStorage>,
    ) -> Result<Arc<CacheItem>, Rejection> {
        match authority_opt {
            Some(authority) => {
                let key = sanitize_path(tail.as_str()).map(|s| {
                    if s.is_empty() {
                        "index.html".to_owned()
                    } else {
                        s
                    }
                })?;
                let host = authority.host();
                if let Some(cache_item) = domain_storage.get_file(host, &key) {
                    Ok(cache_item)
                } else {
                    Err(reject::not_found())
                }
            }
            None => Err(reject::not_found()),
        }
    }
    warp::get()
        .or(warp::head())
        .unify()
        .and(warp::path::tail())
        .and(warp::host::optional())
        .and(with(domain_storage.clone()))
        .and_then(get_cache_file)
        .and(conditionals())
        .map(cache_reply)
    /*
    async fn get_real_path(
        p: warp::path::Tail,
        host: Option<Authority>,
        domain_storage: Arc<DomainStorage>,
    ) -> Result<ArcPath, Rejection> {
        match host {
            Some(h) => {
                if let Some(prefix) = domain_storage.get_version_path(h.as_str()) {
                    let tail = p.as_str();
                    let file = warp::fs::sanitize_path(prefix, tail).map(|mut buf| {
                        if tail.is_empty() {
                            buf.push("index.html");
                        }
                        ArcPath(Arc::new(buf))
                    });
                    file
                } else {
                    Err(reject::not_found())
                }
            }
            None => Err(reject::not_found()),
        }
    }

    warp::get()
        .or(warp::head())
        .unify()
        .and(warp::path::tail())
        .and(warp::host::optional())
        .and(with(domain_storage))
        .and_then(get_real_path)
        .and(conditionals())
        .and_then(warp::fs::file_reply)

     */
}

//from warp::fs
fn sanitize_path(tail: &str) -> Result<String, Rejection> {
    if let Ok(p) = percent_decode_str(tail).decode_utf8() {
        for seg in p.split('/') {
            if seg.starts_with("..") {
                return Err(reject::not_found());
            } else if seg.contains('\\') {
                return Err(reject::not_found());
            }
        }
        Ok(p.into_owned())
    } else {
        Err(reject::not_found())
    }
}
