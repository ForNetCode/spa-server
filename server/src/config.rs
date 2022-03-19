use serde::Deserialize;
use std::env;

const CONFIG_PATH: &str = "config.conf";

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub port: u32,
    pub addr: String,
    pub file_dir: String,
    pub admin_config: Option<AdminConfig>,
    pub https: Option<HttpsConfig>,
    #[serde(default)]
    pub cache: CacheConfig,
}

//TODO: create config with lots of default value
impl Config {
    pub fn load() -> Self {
        let config_path = env::var("SPA_CONFIG").unwrap_or(CONFIG_PATH.to_string());
        //hocon::HoconLoader::load_file(&config_path);

        hocon::HoconLoader::new()
            .load_file(&config_path)
            .expect("can not read config file")
            .resolve::<Config>()
            .expect("parse config file error")
        //let config_str = fs::read_to_string(&config_path).expect("can not read config file");

        //hocon::de::from_str::<Config>(&config_str).expect("parse config file error")
    }
}
#[derive(Deserialize, Debug, Clone)]
pub struct AdminConfig {
    pub port: u32,
    pub addr: String,
    pub token: String,
}

// TLS
#[derive(Deserialize, Debug, Clone)]
pub struct HttpsConfig {
    pub private: String,
    pub public: String,
    pub port: i32,
    pub addr: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct CacheConfig {
    pub max_size: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig { max_size: None }
    }
}
