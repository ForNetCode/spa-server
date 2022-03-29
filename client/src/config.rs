use hocon::HoconLoader;
use serde::Deserialize;
use std::path::PathBuf;

const ENV_CONFIG: &str = include_str!("../client_config_env.conf");

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

impl Config {
    pub fn load(config_dir: Option<PathBuf>) -> anyhow::Result<Config> {
        let mut conf_loader = HoconLoader::new();
        conf_loader = conf_loader.load_str(ENV_CONFIG)?;
        if let Some(config_dir) = config_dir {
            conf_loader = conf_loader.load_file(config_dir)?;
        }

        let hocon = conf_loader.hocon()?;
        // Hocon has a problem, new will override old even new is error.
        // need to write config conversion by hand.
        let admin_server = hocon["server"].clone().resolve()?;
        let uploading_config = UploadConfig {
            parallel: hocon["upload"]["parallel"]
                .as_string()
                .map(|x| x.parse::<u32>().ok())
                .flatten()
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
    fn config_load() {
        let c = Config::load(None);
        println!("{:?}", c);
    }
}
