use crate::domain_storage::DomainStorage;
use crate::file_cache::{CacheItem, DataBlock};
use crate::with;
use headers::{
    AcceptRanges, CacheControl, ContentEncoding, ContentLength, ContentRange, ContentType,
    HeaderMapExt, LastModified, Range,
};
use hyper::Body;
use percent_encoding::percent_decode_str;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io;
use warp::fs::{
    bytes_range, conditionals, file_stream, optimal_buf_size, ArcPath, Cond, Conditionals,
};
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

// copy from warp::fs file_reply
async fn file_reply(
    item: Arc<CacheItem>,
    path: ArcPath,
    range: Option<Range>,
    modified: Option<LastModified>,
) -> Result<Response<Body>, Rejection> {
    let len = item.meta.len();
    match File::open(path.clone()).await {
        Ok(file) => {
            let resp = bytes_range(range, len)
                .map(|(start, end)| {
                    let sub_len = end - start;
                    let buf_size = optimal_buf_size(&item.meta);
                    let stream = file_stream(file, buf_size, (start, end));
                    let body = Body::wrap_stream(stream);
                    let mut resp = Response::new(body);
                    cache_item_to_response_header(&mut resp, item, modified);
                    if sub_len != len {
                        *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
                        resp.headers_mut().typed_insert(
                            ContentRange::bytes(start..end, len).expect("valid ContentRange"),
                        );
                        resp.headers_mut().typed_insert(ContentLength(sub_len));
                    } else {
                        resp.headers_mut().typed_insert(ContentLength(len));
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
                    tracing::warn!("file permission denied: {:?}", path.as_ref().display());
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
async fn cache_or_file_reply(
    item: Arc<CacheItem>,
    conditionals: Conditionals,
    accept_encoding: Option<String>,
) -> Result<Response<Body>, Rejection> {
    let modified = item.meta.modified().map(LastModified::from).ok();

    let resp = match conditionals.check(modified) {
        Cond::NoBody(resp) => Ok(resp),
        Cond::WithBody(range) => match &item.data {
            DataBlock::CacheBlock {
                bytes,
                compressed,
                path,
            } => {
                if accept_encoding.filter(|x| x.contains("gzip")).is_none() && *compressed {
                    file_reply(item.clone(), path.clone(), range, modified).await
                } else {
                    let mut resp = Response::new(Body::from(bytes.clone()));
                    cache_item_to_response_header(&mut resp, item.clone(), modified);
                    resp.headers_mut()
                        .typed_insert(ContentLength(bytes.len() as u64));
                    if *compressed {
                        resp.headers_mut()
                            .typed_insert(ContentLength(bytes.len() as u64));
                        resp.headers_mut().typed_insert(ContentEncoding::gzip());
                    }
                    Ok(resp)
                }
            }
            DataBlock::FileBlock(path) => {
                file_reply(item.clone(), path.clone(), range, modified).await
            }
        },
    };
    resp
}

fn cache_item_to_response_header(
    resp: &mut Response<Body>,
    item: Arc<CacheItem>,
    modified: Option<LastModified>,
) {
    let mime = item.mime.clone();
    resp.headers_mut().typed_insert(ContentType::from(mime));
    resp.headers_mut().typed_insert(AcceptRanges::bytes());
    if let Some(expire) = item.expire {
        if !expire.is_zero() {
            resp.headers_mut()
                .typed_insert(CacheControl::new().with_max_age(expire));
            //for
            //resp.headers_mut()
            //    .typed_insert(Expires::from(SystemTime::now().add(expire)));
        } else {
            resp.headers_mut()
                .typed_insert(CacheControl::new().with_no_cache());
        }
    }
    if let Some(last_modified) = modified {
        resp.headers_mut().typed_insert(last_modified);
    }
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
        .and(warp::header::optional::<String>("accept-encoding"))
        .and_then(cache_or_file_reply)
}
