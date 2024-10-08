use anyhow::{bail, Context};
use duration_str::deserialize_duration;
use serde::{Deserialize, Deserializer};
use small_acme::LetsEncrypt;
use std::time::Duration;
use std::{env, fs};
use std::collections::HashSet;
use headers::{HeaderValue, Origin};
use tracing::warn;

const CONFIG_PATH: &str = "/config/config.toml";

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub file_dir: String,
    #[serde(default)]
    pub cors: Option<HashSet<OriginWrapper>>,
    pub admin_config: Option<AdminConfig>,
    pub http: Option<HttpConfig>,
    pub https: Option<HttpsConfig>,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub domains: Vec<DomainConfig>,
    pub open_telemetry: Option<OpenTelemetry>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct OpenTelemetry {
    pub endpoint: String,
}

//TODO: create config with lots of default value
impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = env::var("SPA_CONFIG").unwrap_or(CONFIG_PATH.to_string());

        let config = Self::load_toml(&config_path)?;
        if config.http.is_none() && config.https.is_none() {
            bail!("should set http or https server config")
        }
        if let Some(http_config) = &config.https {
            if http_config.acme.is_some() && http_config.ssl.is_some() {
                bail!("spa-server don't support ssl and acme config in the meantime");
            }
            if http_config
                .acme
                .as_ref()
                .is_some_and(|c| c.emails.is_empty())
            {
                bail!("acme emails must be set")
            }
            if http_config
                .acme
                .as_ref()
                .is_some_and(|c| matches!(c.acme_type, ACMEType::CI) && c.ci_ca_path.is_none())
            {
                bail!("acme CI must set ca path")
            }
            if http_config.acme.is_some() && config.http.as_ref().filter(|v| v.port != 80).is_none()
            {
                warn!("acme needs http port:80 to signed https certificate");
            }
        }
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
    pub port: u32,
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
    pub cache: Option<DomainCacheConfig>,
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

#[derive(Clone, Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ACMEType {
    CI,
    #[default]
    Prod,
    Stage,
}

impl ACMEType {
    pub fn url(&self) -> &'static str {
        match self {
            ACMEType::Stage => LetsEncrypt::Staging.url(),
            ACMEType::Prod => LetsEncrypt::Production.url(),
            ACMEType::CI => "https://localhost:14000/dir",
        }
    }
    //acme_async regex use this.
    pub fn as_str(&self) -> &'static str {
        match self {
            ACMEType::Stage => "stage",
            ACMEType::Prod => "prod",
            ACMEType::CI => "ci",
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ACMEConfig {
    pub emails: Vec<String>,
    pub dir: Option<String>,
    #[serde(default, rename = "type")]
    pub acme_type: ACMEType,
    pub ci_ca_path: Option<String>, // this is for CI
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct HttpConfig {
    pub addr: String,
    pub port: u16,
    pub external_port: Option<u16>,
    pub redirect_https: Option<bool>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct HttpsConfig {
    pub ssl: Option<SSL>,
    pub acme: Option<ACMEConfig>,
    pub port: u16,
    pub external_port: Option<u16>,
    pub addr: String,
    #[serde(default)]
    pub http_redirect_to_https: u16,
}
// should write Deserialize by hand.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct CacheConfig {
    #[serde(default = "default_max_size")]
    pub max_size: u64,
    #[serde(default)]
    pub compression: bool,
    #[serde(default)]
    pub client_cache: Vec<ClientCacheItem>,
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct DomainCacheConfig {
    pub max_size: Option<u64>,
    pub compression: Option<bool>,
    pub client_cache: Option<Vec<ClientCacheItem>>,
}

fn default_max_size() -> u64 {
    10 * 1000 * 1000
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ClientCacheItem {
    #[serde(deserialize_with = "deserialize_duration")]
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

pub(crate) fn extract_origin(data:&Option<HashSet<OriginWrapper>>) -> Option<HashSet<HeaderValue>> {
    data.as_ref().map(|set| set.iter().map(|o| o.0.clone()).collect())
}

impl <'de> Deserialize<'de> for OriginWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let data = String::deserialize(deserializer)?;
        let mut parts = data.splitn(2, "://");
        let scheme = parts.next().expect("missing scheme");
        let rest = parts.next().expect("missing scheme");
        let origin = Origin::try_from_parts(scheme, rest, None).expect("invalid Origin");
        
        Ok(OriginWrapper(origin.to_string().parse()
            .expect("Origin is always a valid HeaderValue")))
    }
}

#[cfg(test)]
mod test {
    use std::env;
    use std::path::PathBuf;

    fn get_project_path() -> PathBuf {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = path.parent().unwrap();
        path.to_owned()
    }
}
