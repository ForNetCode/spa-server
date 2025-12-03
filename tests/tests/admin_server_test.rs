#![allow(unused_variables)]
mod common;
use common::*;

use spa_server::config::get_host_path_from_domain;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_wrong_token() {
    let domain = LOCAL_HOST.to_owned() + "/27";
    let domain = &domain;

    clean_web_domain_dir(LOCAL_HOST);

    let server_handle = run_server();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (api, _) = get_client_api("client_config_wrong_token.toml");
    let result = api
        .get_domain_info(Some(get_host_path_from_domain(domain).0.to_string()))
        .await;
    assert!(result.is_err());
    server_handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn test_remove_last_version_and_domain() {
    let domain = LOCAL_HOST.to_owned() + "/27";
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080/27");
    let request_prefix = &request_prefix;

    clean_web_domain_dir(LOCAL_HOST);

    let server_handle = run_server();

    tokio::time::sleep(Duration::from_secs(1)).await;

    upload_file_and_check(domain, request_prefix, 1, vec![]).await;

    let (api, _) = get_client_api("client_config.toml");
    api.remove_files(Some(domain.to_string()), Some(1))
        .await
        .unwrap();

    let result = api
        .get_domain_info(Some(get_host_path_from_domain(domain).0.to_string()))
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
    server_handle.abort();
}
