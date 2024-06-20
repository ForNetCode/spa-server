use reqwest::redirect::Policy;
use reqwest::{ClientBuilder, StatusCode};
use std::time::Duration;

mod common;
use crate::common::*;
use common::run_server;
use spa_server::LOCAL_HOST;

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn start_server_and_client_upload_file() {
    let domain = LOCAL_HOST.to_owned() + "/27";
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080/27");
    let request_prefix = &request_prefix;

    clean_test_dir(LOCAL_HOST);

    run_server();

    tokio::time::sleep(Duration::from_secs(1)).await;

    upload_file_and_check(domain, request_prefix, 1, vec!["", "index.html"]).await;
    assert_index_redirect_correct(request_prefix).await;

    assert_expired(
        request_prefix,
        vec![
            ("1.html", Some(0)),
            ("test.js", Some(30 * 24 * 60 * 60)),
            ("test.bin", None),
        ],
    )
    .await;

    upload_file_and_check(domain, request_prefix, 2, vec!["index.html", "2.html"]).await;

    assert_files(domain, request_prefix, 1, vec!["1.html"]).await;

    let (api, _) = get_client_api("client_config.conf");
    api.remove_files(Some(domain.to_string()), Some(1))
        .await
        .unwrap();

    assert_files_no_exists(request_prefix, vec!["1.html"]).await;
    assert_files(domain, request_prefix, 2, vec!["index.html", "2.html"]).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn start_server_with_single_domain() {
    let domain = LOCAL_HOST.to_owned();
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080");
    let request_prefix = &request_prefix;

    clean_test_dir(LOCAL_HOST);

    run_server();

    tokio::time::sleep(Duration::from_secs(2)).await;

    // assert_index_redirect_correct(request_prefix).await;  // http client would auto patch / to http://www.example.com => http://www.example.com/
    upload_file_and_check(domain, request_prefix, 1, vec!["", "index.html"]).await;

    assert_expired(
        request_prefix,
        vec![
            ("1.html", Some(0)),
            ("test.js", Some(30 * 24 * 60 * 60)),
            ("test.bin", None),
        ],
    )
    .await;

    upload_file_and_check(domain, request_prefix, 2, vec!["index.html", "2.html"]).await;

    assert_files(domain, request_prefix, 1, vec!["1.html"]).await;

    let (api, _) = get_client_api("client_config.conf");
    api.remove_files(Some(domain.to_string()), Some(1))
        .await
        .unwrap();

    assert_files_no_exists(request_prefix, vec!["1.html"]).await;
    assert_files(domain, request_prefix, 2, vec!["index.html", "2.html"]).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn multiple_domain_check() {
    clean_test_dir(LOCAL_HOST);
    let domain = format!("{LOCAL_HOST}/27");
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080/27");
    let request_prefix = &request_prefix;

    let domain2 = format!("{LOCAL_HOST}/a");
    let domain2 = &domain2;
    let request_prefix2 = format!("http://{LOCAL_HOST}:8080/a");
    let request_prefix2 = &request_prefix2;

    run_server();

    tokio::time::sleep(Duration::from_secs(1)).await;

    upload_file_and_check(domain, request_prefix, 1, vec!["index.html"]).await;

    upload_file_and_check(domain2, request_prefix2, 1, vec!["index.html"]).await;
    let (api, _) = get_client_api("client_config.conf");
    let result = api.get_domain_info(None).await.unwrap();
    assert_eq!(result.len(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn evoke_cache_when_serving_new_version() {
    clean_test_dir(LOCAL_HOST);
    let domain = format!("{LOCAL_HOST}/27");
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080/27");
    let request_prefix = &request_prefix;
    run_server();

    tokio::time::sleep(Duration::from_secs(1)).await;

    upload_file_and_check(domain, request_prefix, 1, vec!["index.html", "1.html"]).await;
    upload_file_and_check(domain, request_prefix, 2, vec!["index.html", "2.html"]).await;
    upload_file_and_check(domain, request_prefix, 3, vec!["index.html", "3.html"]).await;
    upload_file_and_check(
        domain,
        request_prefix,
        4,
        vec!["index.html", "3.html", "4.html"],
    )
    .await;
    assert_files(domain, request_prefix, 2, vec!["2.html"]).await;
    assert_files_no_exists(request_prefix, vec!["1.html"]).await;
    let (api, _) = get_client_api("client_config.conf");
    let result = api.get_domain_info(None).await.unwrap();
    assert_eq!(result.len(), 1);
}

// This must run after evoke_cache_when_serving_new_index
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn cold_start_server_and_serving_files() {
    let domain = format!("{LOCAL_HOST}/27");
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080/27");
    let request_prefix = &request_prefix;

    run_server();
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_files(domain, request_prefix, 4, vec!["index.html", "4.html"]).await;
    assert_files(domain, request_prefix, 2, vec!["2.html"]).await;
    assert_files_no_exists(request_prefix, vec!["1.html"]).await;
}

#[tokio::test]
async fn simple_hot_reload() {
    clean_test_dir(LOCAL_HOST);
    let domain = format!("{LOCAL_HOST}/27");
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080/27");
    let request_prefix = &request_prefix;

    run_server();
    tokio::time::sleep(Duration::from_secs(2)).await;
    upload_file_and_check(domain, request_prefix, 1, vec!["index.html", "1.html"]).await;
    let src_path = get_template_version(domain, 2);
    let dist_path = get_server_data_path(domain, 2);
    copy_dir_all(src_path, dist_path).unwrap();
    reload_server().await;

    tokio::time::sleep(Duration::from_secs(1)).await;
    upload_file_and_check(domain, request_prefix, 2, vec!["index.html", "2.html"]).await;
}

#[tokio::test]
async fn self_signed_cert_https() {
    clean_test_dir(LOCAL_HOST);
    let domain = format!("{LOCAL_HOST}/27");
    let domain = &domain;
    let request_prefix = format!("https://{LOCAL_HOST}:8443/27");
    let request_prefix = &request_prefix;

    run_server_with_config("server_config_https.conf");
    tokio::time::sleep(Duration::from_secs(2)).await;
    upload_file_and_check(domain, request_prefix, 1, vec!["index.html", "1.html"]).await;
    // TODO: only support 443 port now.
    assert_files(
        domain,
        &format!("http://{LOCAL_HOST}:8080/27"),
        1,
        vec!["index.html", "1.html"],
    )
    .await;
    let req = ClientBuilder::new()
        .redirect(Policy::none())
        .build()
        .unwrap();
    let result = req
        .get(&format!("http://{LOCAL_HOST}:8080/27/index.html"))
        .send()
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::MOVED_PERMANENTLY);
}
