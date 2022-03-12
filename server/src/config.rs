use serde::Deserialize;
use std::env;
use std::fs;

const CONFIG_PATH: &str = "config.conf";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub port: u32,
    pub addr: String,
}

//TODO: create config with lots of default value
impl Config {
    pub fn load() -> Self {
        let config_path = env::var("SPA_CONFIG").unwrap_or(CONFIG_PATH.to_string());
        let config_str = fs::read_to_string(config_path).expect("can not read config file");
        hocon::de::from_str::<Config>(&config_str).expect("parse config file error")
    }
}
