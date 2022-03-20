use hocon::de::wrappers::Serde;
use serde::Deserialize;
use std::env;
use std::time::Duration;

const CONFIG_PATH: &str = "config.conf";

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub port: u32,
    pub addr: String,
    pub file_dir: String,
    #[serde(default)]
    pub cors: bool,
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
    pub http_redirect_to_https: Option<bool>,
}
// should write Deserialize by hand.
#[derive(Deserialize, Debug, Clone)]
pub struct CacheConfig {
    #[serde(default)]
    pub max_size: u64,
    #[serde(default)]
    pub compression: bool,
    #[serde(default)]
    pub client_cache: Vec<ClientCacheItem>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClientCacheItem {
    #[serde(deserialize_with = "Serde::<Duration>::with")]
    pub expire: Duration,
    pub extension_names: Vec<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            max_size: 0,
            client_cache: Vec::new(),
            compression: false,
        }
    }
}
