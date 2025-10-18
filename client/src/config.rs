use anyhow::anyhow;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use toml_edit::DocumentMut;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub server: AdminServerConfig,
    pub upload: UploadConfig,
}
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AdminServerConfig {
    pub address: String,
    pub auth_token: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct UploadConfig {
    pub parallel: u32,
}

fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

impl Config {
    pub fn load(config_dir: Option<PathBuf>) -> anyhow::Result<Config> {
        Self::load_toml(config_dir)
    }
    fn load_toml(config_dir: Option<PathBuf>) -> anyhow::Result<Self> {
        let mut conf = DocumentMut::new();
        if let Some(config_dir) = config_dir {
            let config_str = fs::read_to_string(config_dir)?;
            conf = config_str.parse::<DocumentMut>()?;
        }

        let admin_server = AdminServerConfig {
            address: conf
                .get("server")
                .and_then(|x| x.get("address"))
                .and_then(|x| x.as_str().map(|x| x.to_string()))
                .or_else(|| env_opt("SPA_SERVER_ADDRESS"))
                .ok_or(anyhow!("server.address could not get"))?,
            auth_token: conf
                .get("server")
                .and_then(|x| x.get("auth_token"))
                .and_then(|x| x.as_str().map(|x| x.to_string()))
                .or_else(|| env_opt("SPA_SERVER_AUTH_TOKEN"))
                .ok_or(anyhow!("server.auth_token could not get"))?,
        };
        let uploading_config = UploadConfig {
            parallel: conf
                .get("upload")
                .and_then(|x| x.get("parallel"))
                .and_then(|x| x.as_str().map(|x| x.to_string()))
                .or_else(|| env_opt("SPA_UPLOAD_PARALLEL"))
                .and_then(|x| x.parse::<u32>().ok())
                .unwrap_or(3),
        };
        let config: Config = Config {
            server: admin_server,
            upload: uploading_config,
        };
        Ok(config)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::config::Config;
    use std::env;

    pub(crate) fn init_env() {
        env::set_var("SPA_SERVER_ADDRESS", "http://127.0.0.1:9000");
        env::set_var("SPA_SERVER_AUTH_TOKEN", "token");
        env::set_var("SPA_UPLOAD_PARALLEL", "4");
    }
    fn remove_env() {
        env::remove_var("SPA_SERVER_ADDRESS");
        env::remove_var("SPA_SERVER_AUTH_TOKEN");
        env::remove_var("SPA_UPLOAD_PARALLEL");
    }

    pub(crate) fn default_local_config() -> anyhow::Result<Config> {
        init_env();
        Config::load(None)
    }

    #[test]
    fn config_load_with_env() {
        remove_env();
        //println!("{:?}", default_local_config());
        assert!(default_local_config().is_ok());
    }

    #[test]
    fn config_load() {
        remove_env();
        let c = Config::load(None);
        assert!(c.is_err());
    }
}
