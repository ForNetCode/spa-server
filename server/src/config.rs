use anyhow::Context;
use hocon::de::wrappers::Serde;
use serde::Deserialize;
use std::env;
use std::time::Duration;

const CONFIG_PATH: &str = "config.conf";

// pub type Config = Arc<AppConfig>

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
    #[serde(default)]
    pub domains: Vec<DomainConfig>,
}

//TODO: create config with lots of default value
impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = env::var("SPA_CONFIG").unwrap_or(CONFIG_PATH.to_string());

        let load_file = hocon::HoconLoader::new()
            .load_file(&config_path)
            .with_context(|| "can not read config file")?;

        load_file
            .resolve::<Config>()
            .with_context(|| "parse config file error")
    }
}
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AdminConfig {
    pub port: u32,
    pub addr: String,
    pub token: String,
    #[serde(default = "default_max_upload_size")]
    pub max_upload_size: u64,
}

fn default_max_upload_size() -> u64 {
    30 * 1024 * 1024
}

#[derive(Deserialize, Debug, Clone)]
pub struct DomainConfig {
    pub domain: String,
    pub cors: Option<bool>,
    pub cache: Option<DomainCacheConfig>,
    pub https: Option<DomainHttpsConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DomainHttpsConfig {
    pub ssl: Option<SSL>,
    pub http_redirect_to_https: Option<bool>,
    //#[serde(default)]
    //pub disabled: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SSL {
    pub private: String,
    pub public: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct HttpsConfig {
    pub ssl: Option<SSL>,
    pub port: i32,
    pub addr: String,
    #[serde(default)]
    pub http_redirect_to_https: bool,
}
// should write Deserialize by hand.
#[derive(Deserialize, Debug, Clone)]
pub struct CacheConfig {
    #[serde(default = "default_max_size")]
    pub max_size: u64,
    #[serde(default)]
    pub compression: bool,
    #[serde(default)]
    pub client_cache: Vec<ClientCacheItem>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct DomainCacheConfig {
    pub max_size: Option<u64>,
    pub compression: Option<bool>,
    pub client_cache: Option<Vec<ClientCacheItem>>,
}

fn default_max_size() -> u64 {
    10 * 1024 * 1024
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
            max_size: default_max_size(),
            client_cache: Vec::new(),
            compression: false,
        }
    }
}
