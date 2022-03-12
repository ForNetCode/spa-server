//use hyper::service::Service;
use hyper::header::HOST;
use hyper::{Body, Request, Response};
//use std::task::{Context, Poll};
/*
pub struct StaticFileService {}

impl StaticFileService {
    pub fn new() -> Self {
        StaticFileService {}
    }
}

impl Service<Request<Body>> for StaticFileService {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Result<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let path = req.uri().path();
        Ok(Response::builder().body(Body::from(path)).unwrap())
    }
}*/

pub async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let uri = req.uri();
    let path = uri.path();

    //let host = uri.host();
    info!(
        "look good, {:?}, {}, {:?}",
        uri.scheme(),
        path,
        req.headers().get(HOST)
    );

    Ok(Response::new(Body::from(path.to_owned())))
}
