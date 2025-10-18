use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_stdout::SpanExporter;
use reqwest::header::{CACHE_CONTROL, LOCATION};
use reqwest::redirect::Policy;
use reqwest::{Certificate, Client, ClientBuilder, StatusCode, Url};
use spa_client::api::API;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::{env, fs, io};
//use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, error};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub const LOCAL_HOST: &str = "local.fornetcode.com";
pub const LOCAL_HOST2: &str = "local2.fornetcode.com";

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

/*
fn get_tls_config() -> ClientConfig {
    let mut roots = rustls::RootCertStore::empty();
    let mut reader = io::BufReader::new(
        File::open(get_test_dir().join("pebble/certs/pebble.minica.pem")).unwrap(),
    );
    let cert:Vec<CertificateDer> = rustls_pemfile::certs(&mut reader).map(|v| v.unwrap()).collect();
    roots.add_parsable_certificates(&cert);
    // let mut reader =
    //     io::BufReader::new(File::open(get_test_dir().join("cert/cacerts.pem")).unwrap());
    // let cert = rustls_pemfile::certs(&mut reader).map(|v| v.unwrap());
    // roots.add_parsable_certificates(cert);

    ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth()
}

 */

fn get_root_cert(path: PathBuf) -> Certificate {
    Certificate::from_pem(&fs::read(&path).unwrap()).unwrap()
}
pub fn run_server_with_config(config_file_name: &str) -> JoinHandle<()> {
    env::set_var(
        "SPA_CONFIG",
        get_test_dir().join(config_file_name).display().to_string(),
    );
    let provider = TracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();
    let tracer = provider.tracer("spa-server");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let _ = tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,spa_server=debug,spa_client=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().compact())
        .with(telemetry)
        .try_init();

    // let _ = tracing_subscriber::fmt()
    //     .with_env_filter(
    //         EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "info,spa_server=debug,spa_client=debug".into()),
    //     )
    //     .with_test_writer()
    //     .try_init();
    tokio::spawn(async move {
        let result = spa_server::run_server().await;
        if result.is_err() {
            error!("spa server run error: {:?}", result.unwrap_err());
        } else {
            debug!("spa server finish");
        }
    })
}
pub fn run_server() -> JoinHandle<()> {
    run_server_with_config("server_config.toml")
}

pub async fn reload_server() {
    let client_config =
        spa_client::config::Config::load(Some(get_test_dir().join("client_config.toml"))).unwrap();
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
    let (client_api, client_config) = get_client_api("client_config.toml");
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

pub async fn assert_redirects(request: &str, redirect_urls: Vec<String>) {
    let mut request = request.to_string();
    for redirect_url in redirect_urls {
        let target = assert_redirect_correct(request.as_str(), &redirect_url).await;
        match Url::parse(&target) {
            Ok(_) => {
                request = target;
            }
            Err(_) => {
                let mut url = Url::parse(&request).unwrap();
                url.set_path(&redirect_url);
                request = url.to_string();
            }
        }
    }
}
pub async fn assert_files(
    domain: &str,
    request_prefix: &str,
    version: u32,
    check_path: Vec<&'static str>,
) {
    let client = get_http_client();
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
            println!("begin to check: {request_prefix}/, version:{version}");
            let result = client
                .get(format!("{request_prefix}/"))
                .send()
                .await
                .unwrap();
            assert_eq!(result.status(), StatusCode::OK);
            assert_eq!(
                result.text().await.unwrap(),
                get_file_text(domain, version, file).unwrap()
            );
        }
    }
}
pub fn get_http_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        ClientBuilder::new()
            .add_root_certificate(get_root_cert(
                get_test_dir().join("pebble/certs/pebble.minica.pem"),
            ))
            .tls_built_in_root_certs(false)
            //.use_rustls_tls()
            //.min_tls_version(Version::TLS_1_3)
            //.tls_built_in_root_certs(false)
            //.add_root_certificate(get_root_cert(get_test_dir().join("cert/cacerts.pem")))
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
    })
}
pub fn get_http_no_redirect_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        ClientBuilder::new()
            .add_root_certificate(get_root_cert(
                get_test_dir().join("pebble/certs/pebble.minica.pem"),
            ))
            .tls_built_in_root_certs(false)
            //.use_rustls_tls()
            //.min_tls_version(Version::TLS_1_3)
            //.add_root_certificate(get_root_cert(get_test_dir().join("cert/cacerts.pem")))
            //.tls_built_in_root_certs(false)
            .danger_accept_invalid_certs(true)
            .redirect(Policy::none())
            .build()
            .unwrap()
    })
}

pub async fn assert_redirect_correct(request_prefix: &str, target_prefix: &str) -> String {
    let client = get_http_no_redirect_client();
    let query = [("lang", "rust"), ("browser", "servo"), ("zh", "转义字符")];
    let url = Url::parse_with_params(request_prefix, &query).unwrap();
    let query = url.query().unwrap();
    let response = client.get(url.clone()).send().await.unwrap();

    let location = response
        .headers()
        .get(LOCATION)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
    assert_eq!(
        location,
        format!("{target_prefix}?{query}") //format!("{path}/?{query}")
    );
    location
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

pub fn clean_web_domain_dir(domain: &str) {
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
