use spa_client::run;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .init();
    let result = run().await;
    if let Some(err) = result.err() {
        eprintln!("{}", err);
        std::process::exit(-1);
    }
}
