use anyhow::anyhow;
use hocon::HoconLoader;
use serde::Deserialize;
use std::path::PathBuf;

//const ENV_CONFIG: &str = include_str!("../client_config_env.conf");

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: AdminServerConfig,
    pub upload: UploadConfig,
}
#[derive(Debug, Deserialize, Clone)]
pub struct AdminServerConfig {
    pub address: String,
    pub auth_token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UploadConfig {
    pub parallel: u32,
}

fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

impl Config {
    pub fn load(config_dir: Option<PathBuf>) -> anyhow::Result<Config> {
        let mut conf_loader = HoconLoader::new();

        let config_dir = config_dir.or_else(|| {
            std::env::current_exe().ok().and_then(|p| {
                p.parent().and_then(|p| {
                    let p = p.join("config.conf");
                    if p.exists() {
                        Some(p)
                    } else {
                        None
                    }
                })
            })
        });

        if let Some(config_dir) = config_dir {
            conf_loader = conf_loader.load_file(config_dir)?;
        }

        let hocon = conf_loader.hocon()?;
        // Hocon has a problem, new will override old even new is error.
        // and environment variable will be empty string if not exists.
        let admin_server = AdminServerConfig {
            address: hocon["server"]["address"]
                .as_string()
                .or(env_opt("SPA_SERVER_ADDRESS"))
                .ok_or(anyhow!("server.address could not get"))?,
            auth_token: hocon["server"]["auth_token"]
                .as_string()
                .or(env_opt("SPA_SERVER_AUTH_TOKEN"))
                .ok_or(anyhow!("server.auth_token could not get"))?,
        };
        let uploading_config = UploadConfig {
            parallel: hocon["upload"]["parallel"]
                .as_string()
                .or(env_opt("SPA_UPLOAD_PARALLEL"))
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

    pub(crate) fn default_local_config() -> anyhow::Result<Config> {
        init_env();
        Config::load(None)
    }

    #[test]
    fn config_load_with_env() {
        println!("{:?}", default_local_config());
        //assert!(default_local_config().is_ok());
    }

    #[test]
    fn config_load() {
        let c = Config::load(None);
        println!("{:?}", c);
    }
}
