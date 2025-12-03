#![allow(unused_variables)]
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

mod common;
use crate::common::*;
use common::run_server;
use spa_server::config::get_host_path_from_domain;

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn start_server_and_client_upload_file() {
    let domain = LOCAL_HOST.to_owned() + "/27";
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080/27");
    let request_prefix = &request_prefix;

    clean_web_domain_dir(LOCAL_HOST);

    run_server();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // upload_file_and_check(domain, request_prefix, 1, vec!["", "index.html"]).await;
    upload_file_and_check(domain, request_prefix, 1, vec![]).await;
    assert_redirect_correct(request_prefix, "/27/").await;

    // assert_expired(
    //     request_prefix,
    //     vec![
    //         ("1.html", Some(0)),
    //         ("test.js", Some(30 * 24 * 60 * 60)),
    //         ("test.bin", None),
    //     ],
    // )
    // .await;

    upload_file_and_check(domain, request_prefix, 2, vec!["index.html", "2.html"]).await;

    assert_files(domain, request_prefix, 1, vec!["1.html"]).await;

    let (api, _) = get_client_api("client_config.toml");
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

    clean_web_domain_dir(LOCAL_HOST);

    run_server();

    tokio::time::sleep(Duration::from_secs(1)).await;

    // assert_index_redirect_correct(request_prefix).await;  // http client would auto patch / to http://www.example.com => http://www.example.com/
    upload_file_and_check(domain, request_prefix, 1, vec!["", "index.html"]).await;

    /*
    assert_expired(
        request_prefix,
        vec![
            ("1.html", Some(0)),
            ("test.js", Some(30 * 24 * 60 * 60)),
            ("test.bin", None),
        ],
    )
    .await;
    */
    upload_file_and_check(domain, request_prefix, 2, vec!["index.html", "2.html"]).await;

    assert_files(domain, request_prefix, 1, vec!["1.html"]).await;

    let (api, _) = get_client_api("client_config.toml");
    api.remove_files(Some(domain.to_string()), Some(1))
        .await
        .unwrap();

    assert_files_no_exists(request_prefix, vec!["1.html"]).await;
    assert_files(domain, request_prefix, 2, vec!["index.html", "2.html"]).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn multiple_domain_check() {
    clean_web_domain_dir(LOCAL_HOST);
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
    let (api, _) = get_client_api("client_config.toml");
    let result = api.get_domain_info(None).await.unwrap();
    assert_eq!(result.len(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn revoke_cache_when_serving_new_version() {
    clean_web_domain_dir(LOCAL_HOST);
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
    let (api, _) = get_client_api("client_config.toml");
    let result = api.get_domain_info(None).await.unwrap();
    assert_eq!(result.len(), 1);
}

// This must run after evoke_cache_when_serving_new_index
#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn cold_start_server_and_serving_files() {
    clean_web_domain_dir(LOCAL_HOST);
    let domain = format!("{LOCAL_HOST}/27");
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080/27");
    let request_prefix = &request_prefix;
    let sender = run_server();

    tokio::time::sleep(Duration::from_secs(1)).await;

    upload_file_and_check(domain, request_prefix, 1, vec![]).await;
    upload_file_and_check(domain, request_prefix, 2, vec![]).await;
    upload_file_and_check(domain, request_prefix, 3, vec![]).await;
    upload_file_and_check(domain, request_prefix, 4, vec![]).await;
    let mut wait_count = 0;
    sender.abort();

    debug!("begin to loop server close");

    let (api, _) = get_client_api("client_config.toml");
    loop {
        assert!(wait_count < 10, "10 seconds server does not stop");
        sleep(Duration::from_secs(1)).await;
        let cert_info = api
            .get_domain_info(Some(get_host_path_from_domain(domain).0.to_string()))
            .await;
        if cert_info.is_err() {
            break;
        }
        wait_count += 1;
    }
    let domain = format!("{LOCAL_HOST}/27");
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080/27");
    let request_prefix = &request_prefix;

    run_server();
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_files(domain, request_prefix, 4, vec!["index.html", "4.html"]).await;
    assert_files(domain, request_prefix, 2, vec!["2.html"]).await;
    assert_files_no_exists(request_prefix, vec!["1.html"]).await;
    //sender.send(()).unwrap();
}

#[tokio::test]
async fn single_domain_reject_multiple_update() {
    let domain = LOCAL_HOST.to_owned();
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080");
    let request_prefix = &request_prefix;
    clean_web_domain_dir(LOCAL_HOST);
    run_server();
    tokio::time::sleep(Duration::from_secs(1)).await;
    upload_file_and_check(domain, request_prefix, 1, vec![]).await;

    let domain = format!("{LOCAL_HOST}/27");
    let domain = &domain;

    let (client_api, client_config) = get_client_api("client_config.toml");

    let upload_result = spa_client::upload_files(
        client_api.clone(),
        domain.to_string(),
        None,
        get_template_version(domain, 1),
        client_config.upload.parallel,
    )
    .await;
    assert!(upload_result.is_err());
}

#[tokio::test]
async fn multiple_domain_reject_single_update() {
    let domain = format!("{LOCAL_HOST}/27");
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}/27:8080");
    let request_prefix = &request_prefix;
    clean_web_domain_dir(LOCAL_HOST);
    run_server();
    tokio::time::sleep(Duration::from_secs(1)).await;
    upload_file_and_check(domain, request_prefix, 1, vec![]).await;

    let domain = LOCAL_HOST.to_string();
    let domain = &domain;

    let (client_api, client_config) = get_client_api("client_config.toml");

    let upload_result = spa_client::upload_files(
        client_api.clone(),
        domain.to_string(),
        None,
        get_template_version(domain, 1),
        client_config.upload.parallel,
    )
    .await;
    assert!(upload_result.is_err());
}

#[tokio::test]
async fn revoke_version() {
    clean_web_domain_dir(LOCAL_HOST);
    let domain = format!("{LOCAL_HOST}/27");
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST}:8080/27");
    let request_prefix = &request_prefix;
    run_server();

    tokio::time::sleep(Duration::from_secs(1)).await;

    upload_file_and_check(domain, request_prefix, 1, vec![]).await;
    upload_file_and_check(domain, request_prefix, 2, vec![]).await;
    upload_file_and_check(domain, request_prefix, 3, vec![]).await;
    let (api, _) = get_client_api("client_config.toml");
    api.revoke_version(domain.to_string(), 2).await.unwrap();

    assert_files(domain, request_prefix, 2, vec!["index.html", "2.html"]).await;
    assert_files_no_exists(request_prefix, vec!["3.html"]).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn alias_start_server_and_client_upload_file() {
    let domain = LOCAL_HOST.to_owned() + "/27";
    let domain = &domain;
    let request_prefix = format!("http://{LOCAL_HOST2}:8080/27");
    let request_prefix = &request_prefix;

    clean_web_domain_dir(LOCAL_HOST);
    run_server_with_config("server_config_alias.toml");
    tokio::time::sleep(Duration::from_secs(1)).await;
    upload_file_and_check(domain, request_prefix, 1, vec!["index.html"]).await;
    assert_redirects(request_prefix, vec!["/27/".to_owned()]).await
    // assert_redirects(
    //     request_prefix,
    //     vec![format!("http://{LOCAL_HOST}:8080/27"), "/27/".to_owned()],
    // )
    // .await
}
