use anyhow::Result;
use chrono::Local;
use env_logger::Builder;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
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

    spa_server::Server::new().run().await
}
