use anyhow::{Context, bail};
use duration_str::deserialize_duration;
use salvo::http::HeaderValue;
use serde::{Deserialize, Deserializer};
use std::collections::HashSet;
use std::time::Duration;
use std::{env, fs};

const CONFIG_PATH: &str = "config.toml";

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub file_dir: String,
    #[serde(default)]
    pub cors: Option<HashSet<OriginWrapper>>,
    pub admin_config: Option<AdminConfig>,
    pub http: HttpConfig,
    #[serde(default)]
    pub domains: Vec<DomainConfig>,
}

//TODO: create config with lots of default value
impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = env::var("SPA_CONFIG").unwrap_or(CONFIG_PATH.to_string());

        let config = Self::load_toml(&config_path)?;

        if config
            .domains
            .iter()
            .any(|x| !get_host_path_from_domain(&x.domain).1.is_empty())
        {
            bail!("domains.domain do not support sub path like 'www.example.com/abc' now")
        }
        Ok(config)
    }

    fn load_toml(path: &str) -> anyhow::Result<Self> {
        let config = fs::read_to_string(path)
            .with_context(|| format!("can not read config file: {path}"))?;
        let config: Config = toml::from_str(&config).with_context(|| "parse config file error")?;
        Ok(config)
    }
}
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AdminConfig {
    pub port: u16,
    pub addr: String,
    pub token: String,
    #[serde(default = "default_max_upload_size")]
    pub max_upload_size: u64,
    pub deprecated_version_delete: Option<DeprecatedVersionRemove>,
}

fn default_max_upload_size() -> u64 {
    30 * 1024 * 1024
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct DomainConfig {
    pub domain: String,
    pub cors: Option<HashSet<OriginWrapper>>,
    pub https: Option<DomainHttpsConfig>,
    pub alias: Option<Vec<String>>,
    pub redirect_https: Option<bool>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct DomainHttpsConfig {
    pub ssl: Option<SSL>,
    #[serde(default)]
    pub disable_acme: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct SSL {
    pub private: String,
    pub public: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct HttpConfig {
    pub addr: String,
    pub port: u16,
    pub external_port: Option<u16>,
    pub redirect_https: Option<bool>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ClientCacheItem {
    #[serde(deserialize_with = "deserialize_duration")]
    pub expire: Duration,
    pub extension_names: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct DeprecatedVersionRemove {
    #[serde(default = "default_cron")]
    pub cron: String,
    #[serde(default = "default_max_reserve")]
    pub max_reserve: u32,
}
pub fn default_cron() -> String {
    String::from("0 0 3 * * *")
}
pub fn default_max_reserve() -> u32 {
    2
}

pub fn get_host_path_from_domain(domain: &str) -> (&str, &str) {
    match domain.split_once('/') {
        None => (domain, ""),
        Some(v) => v,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OriginWrapper(HeaderValue);

pub(crate) fn extract_origin(
    data: &Option<HashSet<OriginWrapper>>,
) -> Option<HashSet<HeaderValue>> {
    data.as_ref()
        .map(|set| set.iter().map(|o| o.0.clone()).collect())
}

impl<'de> Deserialize<'de> for OriginWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = String::deserialize(deserializer)?;
        let mut parts = data.splitn(2, "://");
        let scheme = parts.next().expect("missing scheme");
        let rest = parts.next().expect("missing scheme");
        let origin = salvo::http::headers::Origin::try_from_parts(scheme, rest, None)
            .expect("invalid Origin");

        Ok(OriginWrapper(
            origin
                .to_string()
                .parse()
                .expect("Origin is always a valid HeaderValue"),
        ))
    }
}
