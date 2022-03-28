use anyhow::Result;
use clap::Parser;
use spa_client::commands::CliCommand;
use tracing_subscriber::EnvFilter;
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    Ok(())
}
