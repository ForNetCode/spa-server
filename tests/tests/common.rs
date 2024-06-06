use reqwest::StatusCode;
use std::path::PathBuf;
use std::{env, fs, io};
use tokio::task::JoinHandle;
use tracing::Level;
use tracing_subscriber::EnvFilter;

pub fn get_test_dir() -> PathBuf {
    env::current_dir().unwrap().join("data")
}
pub fn get_template_version(domain: &str, version: u32) -> PathBuf {
    get_test_dir()
        .join("template")
        .join(domain)
        .join(version.to_string())
}
pub fn get_file_text(domain: &str, version: u32, path: &str) -> io::Result<String> {
    let path = get_template_version(domain, version).join(path);
    fs::read_to_string(path)
}

pub fn run_server() -> JoinHandle<()> {
    env::set_var(
        "SPA_CONFIG",
        get_test_dir()
            .join("server_config.conf")
            .display()
            .to_string(),
    );
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::DEBUG.into())
                .from_env_lossy(),
        )
        .init();

    return tokio::spawn(async {
        spa_server::run_server().await.unwrap();
    });
}

pub async fn upload_file_and_check(
    domain: &str,
    request_prefix: &str,
    version: u32,
    check_path: Vec<&'static str>,
) {
    let client_config =
        spa_client::config::Config::load(Some(get_test_dir().join("client_config.conf"))).unwrap();

    println!("begin to upload file");

    let client_api = spa_client::api::API::new(&client_config).unwrap();

    spa_client::upload_files(
        client_api.clone(),
        domain.to_string(),
        None,
        get_template_version(domain, version),
        client_config.upload.parallel,
    )
    .await
    .unwrap();

    let result = client_api
        .release_domain_version(domain.to_string(), None)
        .await
        .unwrap();

    println!("release result: {result}");

    assert_files(domain, request_prefix, version, check_path).await;
}
pub async fn assert_files(
    domain: &str,
    request_prefix: &str,
    version: u32,
    check_path: Vec<&'static str>,
) {
    for file in check_path {
        println!("begin to check: {request_prefix}/{file}, version:{version}");
        let result = reqwest::get(format!("{request_prefix}/{file}"))
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.text().await.unwrap(),
            get_file_text(domain, version, file).unwrap()
        );
    }
}
pub async fn assert_files_no_exists(request_prefix: &str, check_path: Vec<&'static str>) {
    for file in check_path {
        println!("begin to check: {request_prefix}/{file} no exists");
        assert_eq!(
            reqwest::get(format!("{request_prefix}/{file}"))
                .await
                .unwrap()
                .status(),
            StatusCode::NOT_FOUND
        );
    }
}
pub fn clean_test_dir(domain: &str) {
    let path = get_test_dir().join("web").join(domain);
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }
}
