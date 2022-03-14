use chrono::Local;
use env_logger::Builder;
use spa_server::admin_server::AdminServer;
use spa_server::config::Config;
use spa_server::domain_storage::DomainStorage;
use std::io::Write;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}] {}",
                Local::now().format("%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
    let config = Config::load();
    let domain_storage = Arc::new(DomainStorage::init(&config.file_dir)?);
    let server = AdminServer::new(config.admin_config.unwrap(), domain_storage);
    server.run().await?;
    Ok(())
}
