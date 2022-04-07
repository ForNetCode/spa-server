use spa_server::config::Config;
use spa_server::tls::load_ssl_server_config;
use std::env;
use std::fs::File;
use std::path::PathBuf;

#[test]
pub fn load_test() {
    env::set_var("SPA_CONFIG", "tests/data/test1.conf");
    let config = Config::load().unwrap();
    load_ssl_server_config(&config).unwrap();
}
