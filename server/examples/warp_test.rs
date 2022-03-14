use std::path::{Path, PathBuf};
use std::sync::Arc;
use warp::fs::conditionals;
use warp::host::Authority;
use warp::Filter;

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    //let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    warp::fs::dir("");
    let file = warp::get()
        .or(warp::head())
        .unify()
        .and(warp::path::tail())
        .and(warp::host::optional())
        .map(|p: warp::path::Tail, host: Option<Authority>| {
            let arc_path = warp::fs::ArcPath(Arc::new(PathBuf::from(
                "/Users/timzaak/code/work/self/spa-server/tmp/self.noti.link/1/index.html",
            )));
            //let real_path = warp::fs::sanitize_path(arc_path,p.as_str());
            arc_path
        })
        .and(conditionals())
        .and_then(warp::fs::file_reply);

    warp::serve(file).run(([127, 0, 0, 1], 3030)).await;
}
