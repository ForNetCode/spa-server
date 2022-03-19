use crate::domain_storage::DomainStorage;
use crate::file_cache::{CacheItem, DataBlock};
use crate::with;
use headers::{AcceptRanges, ContentLength, ContentRange, ContentType, HeaderMapExt, LastModified};
use hyper::Body;
use percent_encoding::percent_decode_str;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io;
use warp::fs::{bytes_range, conditionals, file_stream, optimal_buf_size, Cond, Conditionals};
use warp::host::Authority;
use warp::http::{Response, StatusCode};
use warp::{reject, Filter, Rejection};

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

async fn cache_reply(
    item: Arc<CacheItem>,
    conditionals: Conditionals,
) -> Result<Response<Body>, Rejection> {
    let modified = item.meta.modified().map(LastModified::from).ok();
    let mut len = item.meta.len();

    let resp = match conditionals.check(modified) {
        Cond::NoBody(resp) => Ok(resp),
        Cond::WithBody(range) => match &item.data {
            DataBlock::CacheBlock(bytes) => {
                let mut resp = Response::new(Body::from(bytes.clone()));
                resp.headers_mut().typed_insert(ContentLength(len));
                resp.headers_mut()
                    .typed_insert(ContentType::from(item.mime.clone()));
                resp.headers_mut().typed_insert(AcceptRanges::bytes());
                if let Some(last_modified) = modified {
                    resp.headers_mut().typed_insert(last_modified);
                }
                Ok(resp)
            }
            DataBlock::FileBlock(path) => {
                // copy from warp::fs file_replyZ
                match File::open(path.clone()).await {
                    Ok(file) => {
                        let resp = bytes_range(range, len)
                            .map(|(start, end)| {
                                let sub_len = end - start;
                                let buf_size = optimal_buf_size(&item.meta);
                                let stream = file_stream(file, buf_size, (start, end));
                                let body = Body::wrap_stream(stream);
                                let mut resp = Response::new(body);
                                if sub_len != len {
                                    *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
                                    resp.headers_mut().typed_insert(
                                        ContentRange::bytes(start..end, len)
                                            .expect("valid ContentRange"),
                                    );
                                    len = sub_len;
                                }
                                let mime = item.mime.clone();
                                resp.headers_mut().typed_insert(ContentLength(len));
                                resp.headers_mut().typed_insert(ContentType::from(mime));
                                resp.headers_mut().typed_insert(AcceptRanges::bytes());

                                if let Some(last_modified) = modified {
                                    resp.headers_mut().typed_insert(last_modified);
                                }
                                resp
                            })
                            .unwrap_or_else(|_| {
                                // bad byte range
                                let mut resp = Response::new(Body::empty());
                                *resp.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
                                resp.headers_mut()
                                    .typed_insert(ContentRange::unsatisfied_bytes(len));
                                resp
                            });
                        Ok(resp)
                    }
                    Err(err) => {
                        let rej = match err.kind() {
                            io::ErrorKind::NotFound => {
                                tracing::debug!("file not found: {:?}", path.as_ref().display());
                                reject::not_found()
                            }
                            io::ErrorKind::PermissionDenied => {
                                tracing::warn!(
                                    "file permission denied: {:?}",
                                    path.as_ref().display()
                                );
                                reject::not_found()
                                //reject::known(FilePermissionError { _p: () })
                            }
                            _ => {
                                tracing::error!(
                                    "file open error (path={:?}): {} ",
                                    path.as_ref().display(),
                                    err
                                );
                                reject::not_found()
                            }
                        };
                        Err(rej)
                    }
                }
            }
        },
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
        .and_then(cache_reply)
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
