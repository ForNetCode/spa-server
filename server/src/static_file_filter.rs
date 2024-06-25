use crate::domain_storage::DomainStorage;
use crate::file_cache::{CacheItem, DataBlock};
use crate::service::not_found;
use headers::{
    AcceptRanges, CacheControl, ContentEncoding, ContentLength, ContentRange, ContentType,
    HeaderMapExt, LastModified, Range,
};
use hyper::body::Bytes;
use hyper::Body;
use percent_encoding::percent_decode_str;
use std::convert::Infallible;
use std::ops::Bound;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io;
use tracing::debug;
use warp::fs::{file_stream, optimal_buf_size, Cond, Conditionals};
use warp::http::{Response, StatusCode};

//from warp::fs
fn sanitize_path(tail: &str) -> Option<String> {
    if let Ok(p) = percent_decode_str(tail).decode_utf8() {
        for seg in p.split('/') {
            if seg.starts_with("..") || seg.contains('\\') {
                return None
            }
        }
        Some(p.into_owned())
    } else {
        None
    }
}
#[derive(Debug)]
pub struct BadRange;

//from warp/fs
fn bytes_range(range: Option<Range>, max_len: u64) -> Result<(u64, u64), BadRange> {
    let range = if let Some(range) = range {
        range
    } else {
        return Ok((0, max_len));
    };

    let ret = range
        .iter()
        .map(|(start, end)| {
            let start = match start {
                Bound::Unbounded => 0,
                Bound::Included(s) => s,
                Bound::Excluded(s) => s + 1,
            };

            let end = match end {
                Bound::Unbounded => max_len,
                Bound::Included(s) => {
                    // For the special case where s == the file size
                    if s == max_len {
                        s
                    } else {
                        s + 1
                    }
                }
                Bound::Excluded(s) => s,
            };

            if start < end && end <= max_len {
                Ok((start, end))
            } else {
                tracing::trace!("unsatisfiable byte range: {}-{}/{}", start, end, max_len);
                Err(BadRange)
            }
        })
        .next()
        .unwrap_or(Ok((0, max_len)));
    ret
}

// copy from warp::fs file_reply
async fn file_reply(
    item: &CacheItem,
    path: &Path,
    range: Option<Range>,
    modified: Option<LastModified>,
) -> Response<Body> {
    let len = item.meta.len();
    match File::open(path).await {
        Ok(file) => {
            let resp = bytes_range(range, len)
                .map(|(start, end)| {
                    let sub_len = end - start;
                    let buf_size = optimal_buf_size(&item.meta);
                    let stream = file_stream(file, buf_size, (start, end));
                    let body = Body::wrap_stream(stream);
                    let mut resp = Response::new(body);
                    cache_item_to_response_header(&mut resp, &item, modified);
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
            resp
        }
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    debug!("file not found: {:?}", path.display());
                }
                io::ErrorKind::PermissionDenied => {
                    tracing::warn!("file permission denied: {:?}", path.display());
                    //reject::known(FilePermissionError { _p: () })
                }
                _ => {
                    tracing::error!("file open error (path={:?}): {} ", path.display(), err);
                }
            };
            not_found()
        }
    }
}

fn cache_reply(
    item: &CacheItem,
    bytes: &Bytes,
    range: Option<Range>,
    modified: Option<LastModified>,
) -> Response<Body> {
    let mut len = bytes.len() as u64;
    // don't support multiple range
    bytes_range(range, len)
        .map(|(start, end)| {
            let sub_len = end - start;
            // range or all
            let body = Body::from(bytes.slice((
                Bound::Included(start as usize),
                Bound::Excluded(end as usize),
            )));
            let mut resp = Response::new(body);

            if sub_len != len {
                *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
                resp.headers_mut().typed_insert(
                    ContentRange::bytes(start..end, len).expect("valid ContentRange"),
                );
                len = sub_len;
                resp.headers_mut().typed_insert(ContentLength(sub_len));
            } else {
                resp.headers_mut().typed_insert(ContentLength(len));
            }
            cache_item_to_response_header(&mut resp, &item, modified);
            resp
        })
        .unwrap_or_else(|_| {
            let mut resp = Response::new(Body::empty());
            *resp.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
            resp.headers_mut()
                .typed_insert(ContentRange::unsatisfied_bytes(len));
            resp
        })
}

pub async fn cache_or_file_reply(
    item: (String, Arc<CacheItem>),
    conditionals: Conditionals,
    accept_encoding: Option<String>,
) -> Result<Response<Body>, Infallible> {
    let (key, item) = item;
    let modified = item.meta.modified().map(LastModified::from).ok();
    match conditionals.check(modified) {
        Cond::NoBody(resp) => {
            debug!("{} hit client cache", key);
            Ok(resp)
        }
        Cond::WithBody(range) => match &item.data {
            DataBlock::CacheBlock {
                bytes,
                path,
                compressed,
            } => {
                let compressed = *compressed;
                let client_accept_gzip = accept_encoding
                    .as_ref()
                    .filter(|x| x.contains("gzip"))
                    .is_some();
                //gzip header, compressed_data
                //true,true => cache
                //true, false => cache without content-encoding
                //false,false => cache without content-encoding
                //false, true => file
                if !client_accept_gzip && compressed {
                    debug!("{} hit disk", key);
                    Ok(file_reply(&item, path.as_ref(), range, modified).await)
                } else {
                    let mut resp = cache_reply(item.as_ref(), bytes, range, modified);
                    if client_accept_gzip && compressed {
                        debug!("{} hit cache, compressed", key);
                        resp.headers_mut().typed_insert(ContentEncoding::gzip());
                    } else {
                        debug!("{} hit cache", key);
                    }
                    Ok(resp)
                }
            }
            DataBlock::FileBlock(path) => {
                debug!("{} hit disk", key);
                Ok(file_reply(&item, path.as_ref(), range, modified).await)
            }
        },
    }
}

fn cache_item_to_response_header(
    resp: &mut Response<Body>,
    item: &CacheItem,
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

pub async fn get_cache_file(
    tail: &str,
    host: &str,
    domain_storage: Arc<DomainStorage>,
) -> Option<(String, Arc<CacheItem>)> {
    let _key = sanitize_path(tail)?;
    let key = _key[1..].to_owned();
    debug!("get file: {host}, tail:{_key}, fixed: {key}");
    if let Some(cache_item) = domain_storage.get_file(host, &key) { 
        Some((key, cache_item))
    } else {
        debug!("no file for: {}/{}", &host, &key);
        None
    }
}
