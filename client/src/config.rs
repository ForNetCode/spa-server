const CONFIG_FILE: &str = "client_config.conf";

#[derive(Debug)]
pub struct Config {
    parallel: u32,
    server_address: String,
    server_auth_token: String,
}
impl Config {}
