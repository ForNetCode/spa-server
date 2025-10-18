mod otlp;

use spa_server::config::Config;
use tracing_core::Level;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    let open_telemetry = &config.open_telemetry;
    if let Some(open_telemetry) = open_telemetry.as_ref() {
        otlp::init_otlp(open_telemetry.endpoint.clone())?;
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(Level::INFO.into())
                    .from_env_lossy(),
            )
            .init();
    }

    tracing::debug!("config load:{:?}", &config);
    spa_server::run_server_with_config(config).await
}
