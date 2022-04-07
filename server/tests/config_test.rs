
use spa_server::config::Config;
use std::env;

pub fn get_test1_config() -> Config {
    env::set_var("SPA_CONFIG", "tests/data/test1.conf");
    Config::load().unwrap()
}
#[test]
fn test1_config() {
    let config = get_test1_config();
    assert!(!config.domains.is_empty());
}
