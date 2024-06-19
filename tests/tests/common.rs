use reqwest::header::CACHE_CONTROL;
use reqwest::redirect::Policy;
use reqwest::{ClientBuilder, StatusCode};
use spa_client::api::API;
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use tokio::task::JoinHandle;
use tracing::{error, Level};
use tracing_subscriber::EnvFilter;

pub fn get_test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}
pub fn get_template_version(domain: &str, version: u32) -> PathBuf {
    get_test_dir()
        .join("template")
        .join(domain)
        .join(version.to_string())
}
pub fn get_file_text(domain: &str, version: u32, path: &str) -> io::Result<String> {
    let path = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path
    };
    let path = get_template_version(domain, version).join(path);
    fs::read_to_string(path)
}

pub fn get_server_data_path(domain: &str, version: u32) -> PathBuf {
    get_test_dir()
        .join("web")
        .join(domain)
        .join(version.to_string())
}

pub fn run_server_with_config(config_file_name: &str) -> JoinHandle<()> {
    env::set_var(
        "SPA_CONFIG",
        get_test_dir().join(config_file_name).display().to_string(),
    );
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::DEBUG.into())
                .from_env_lossy(),
        )
        .with_test_writer()
        .try_init();

    return tokio::spawn(async {
        let result = spa_server::run_server().await;
        if result.is_err() {
            error!("spa server run error: {:?}", result.unwrap_err())
        }
    });
}
pub fn run_server() -> JoinHandle<()> {
    run_server_with_config("server_config.conf")
}

pub async fn reload_server() {
    let client_config =
        spa_client::config::Config::load(Some(get_test_dir().join("client_config.conf"))).unwrap();
    let client_api = API::new(&client_config).unwrap();
    client_api.reload_spa_server().await.unwrap()
}

pub fn get_client_api(config_file_name: &str) -> (API, spa_client::config::Config) {
    let client_config =
        spa_client::config::Config::load(Some(get_test_dir().join(config_file_name))).unwrap();
    (API::new(&client_config).unwrap(), client_config)
}

pub async fn upload_file_and_check(
    domain: &str,
    request_prefix: &str,
    version: u32,
    check_path: Vec<&'static str>,
) {
    let (client_api, client_config) = get_client_api("client_config.conf");
    println!("begin to upload file");

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
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .redirect(Policy::default())
        .build()
        .unwrap();
    for file in check_path {
        println!("begin to check: {request_prefix}/{file}, version:{version}");
        let result = client
            .get(format!("{request_prefix}/{file}"))
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result.text().await.unwrap(),
            get_file_text(domain, version, file).unwrap()
        );
        if file.is_empty() {
            println!("begin to check: {request_prefix}, version:{version}");
            let result = client.get(request_prefix).send().await.unwrap();
            assert_eq!(result.status(), StatusCode::OK);
            assert_eq!(
                result.text().await.unwrap(),
                get_file_text(domain, version, file).unwrap()
            );
        }
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

pub async fn assert_expired(request_prefix: &str, check_path: Vec<(&'static str, Option<u64>)>) {
    for (file, expired) in check_path {
        println!("begin to check: {request_prefix}/{file} expired");
        let result = reqwest::get(format!("{request_prefix}/{file}"))
            .await
            .unwrap();

        let cache_option = result.headers().get(CACHE_CONTROL);

        match expired {
            Some(expired) => {
                let mut expect = "no-cache".to_string();
                if expired > 0 {
                    //expect = expect.with_max_age(Duration::from_secs(expired));
                    expect = format!("max-age={expired}");
                }
                assert_eq!(cache_option.unwrap().to_str().unwrap(), &expect);
            }
            None => assert_eq!(cache_option, None),
        }
    }
}

pub fn clean_test_dir(domain: &str) {
    let path = get_test_dir().join("web").join(domain);
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }
}

pub fn copy_dir_all<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dst: P2) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    if !dst.exists() {
        fs::create_dir(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_all(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
        }
    }
    Ok(())
}
