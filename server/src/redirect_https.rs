use crate::DomainStorage;
use hyper::header::LOCATION;
use hyper::http::uri::PathAndQuery;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode, Uri};
use std::net::SocketAddr;
use std::sync::Arc;
use warp::host::Authority;

async fn redirect_https_handler(
    req: Request<Body>,
    storage: Arc<DomainStorage>,
) -> anyhow::Result<Response<Body>> {
    let uri = req.uri();
    let host = req
        .headers()
        .get("host")
        .map(|x| x.to_str().ok())
        .flatten()
        .unwrap_or("");

    if storage.get_domain_info_by_domain(host).is_some() {
        let uri = Uri::builder()
            .scheme("https")
            .authority(host.parse::<Authority>()?)
            .path_and_query(
                uri.path_and_query()
                    .cloned()
                    .unwrap_or(PathAndQuery::from_static("/")),
            )
            .build()
            .unwrap();
        let mut resp = Response::default();

        resp.headers_mut()
            .insert(LOCATION, uri.to_string().parse().unwrap());
        *resp.status_mut() = StatusCode::MOVED_PERMANENTLY;
        Ok(resp)
    } else {
        let mut res = Response::default();
        *res.status_mut() = StatusCode::NOT_FOUND;
        Ok(res)
    }
}

pub async fn hyper_redirect_server(
    addr: SocketAddr,
    storage: Arc<DomainStorage>,
) -> anyhow::Result<()> {
    let service = make_service_fn(|_| {
        let storage = storage.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                redirect_https_handler(req, storage.clone())
            }))
        }
    });
    let server = Server::bind(&addr).serve(service);
    tracing::info!("http redirect to https, listening on http://{}", addr);
    server.await?;
    Ok(())
}
