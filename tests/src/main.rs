use std::env;
use std::path::PathBuf;
use anyhow::Result;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var(
        "SPA_CONFIG",
        get_test_dir().join("server_config.conf").display().to_string(),
    );
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::DEBUG.into())
                .from_env_lossy(),
        )
        .init();
    spa_server::run_server().await
}

pub fn get_test_dir() -> PathBuf {
    env::current_dir().unwrap().join("tests").join("data")
}