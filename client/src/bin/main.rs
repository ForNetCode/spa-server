use tracing_subscriber::EnvFilter;
//use spa_client::run;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    //run();
}
