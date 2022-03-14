use tracing_subscriber;
fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Hello World!!!")
}
