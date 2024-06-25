use crate::service::resp;
use futures_util::future::Either;
use headers::{AccessControlAllowMethods, HeaderMap, HeaderMapExt};
use hyper::http::HeaderValue;
use hyper::{header, Body, Method, Response, StatusCode};
use lazy_static::lazy_static;
use std::collections::HashSet;

pub enum Validated {
    Simple(HeaderValue),
    NotCors,
}
lazy_static! {
    static ref ALLOW_METHODS: HashSet<Method> =
        HashSet::from([Method::GET, Method::OPTIONS, Method::HEAD]);
}

pub fn cors_resp(mut res: Response<Body>, origin: HeaderValue) -> Response<Body> {
    let headers = res.headers_mut();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        HeaderValue::from_static("true"),
    );

    let access_control_allow_methods: AccessControlAllowMethods =
        ALLOW_METHODS.iter().cloned().collect();
    headers.typed_insert(access_control_allow_methods);
    headers.insert(header::ACCESS_CONTROL_MAX_AGE, 3600.into());
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    res
}

pub fn resp_cors_request(
    method: &Method,
    headers: &HeaderMap,
    allow_cors: bool,
) -> Either<Validated, Response<Body>> {
    match (headers.get(header::ORIGIN), method) {
        (Some(origin), &Method::OPTIONS) => {
            if !allow_cors {
                return Either::Right(resp(StatusCode::FORBIDDEN, "origin not allowed"));
            }
            if let Some(req_method) = headers.get(header::ACCESS_CONTROL_REQUEST_METHOD) {
                if !Method::from_bytes(req_method.as_bytes())
                    .map(|method| ALLOW_METHODS.contains(&method))
                    .unwrap_or(false)
                {
                    return Either::Right(resp(
                        StatusCode::FORBIDDEN,
                        "request-method not allowed",
                    ));
                }
            } else {
                tracing::trace!("preflight request missing access-control-request-method header");
                return Either::Right(resp(StatusCode::FORBIDDEN, "request-method not allowed"));
            }
            let res = Response::default();
            let res = cors_resp(res, origin.clone());
            Either::Right(res)
        }
        (Some(origin), _) => {
            if !allow_cors {
                Either::Right(resp(StatusCode::FORBIDDEN, "origin not allowed"))
            } else {
                Either::Left(Validated::Simple(origin.clone()))
            }
        }
        (None, _) => Either::Left(Validated::NotCors),
    }
}
