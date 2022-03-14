use crate::domain_storage::DomainStorage;
use std::sync::Arc;
use warp::fs::{conditionals, ArcPath, File};
use warp::host::Authority;
use warp::{reject, Filter, Rejection};

pub fn static_file_filter(
    domain_storage: Arc<DomainStorage>,
) -> impl Filter<Extract = (File,), Error = Rejection> + Clone {
    async fn get_real_path(
        p: warp::path::Tail,
        host: Option<Authority>,
        domain_storage: Arc<DomainStorage>,
    ) -> Result<ArcPath, Rejection> {
        if let Some(Some(prefix)) = host.map(|h| domain_storage.get_version_path(h.as_str())) {
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

    warp::get()
        .or(warp::head())
        .unify()
        .and(warp::path::tail())
        .and(warp::host::optional())
        .and(warp::any().map(move || domain_storage.clone()))
        .and_then(get_real_path)
        .and(conditionals())
        .and_then(warp::fs::file_reply)
}
