use futures::StreamExt;
use indicatif::ProgressBar;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime;
use tokio::sync::Mutex;
use tokio::time::Instant;

// These code is to find out the right progress to use.
fn main() {
    let steps = 10;
    let pb = ProgressBar::new(steps);

    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(4)
        .build()
        .expect("failed to create runtime");
    let pb = Arc::new(Mutex::new(pb));
    rt.block_on(concurrent_parallel(pb));
}

async fn concurrent_parallel(pb: Arc<Mutex<ProgressBar>>) {
    let before = Instant::now();
    let paths = (0..6).rev();
    let fetches = futures::stream::iter(
        paths
            .into_iter()
            .map(|path| tokio::spawn(make_request(path, pb.clone()))),
    )
    .buffer_unordered(4)
    .collect::<Vec<_>>();
    fetches.await;
    pb.lock().await.finish();
    println!("elapsed time: {:.2?}", before.elapsed());
}
async fn make_request(sleep: u64, pb: Arc<Mutex<ProgressBar>>) -> String {
    std::thread::sleep(Duration::from_secs(sleep));
    let p = pb.lock().await;
    p.inc(1);
    format!(
        "current thread {:?} | thread name {} | request_duration {:?}",
        std::thread::current().id(),
        std::thread::current()
            .name()
            .get_or_insert("default_thread_name"),
        sleep
    )
}
