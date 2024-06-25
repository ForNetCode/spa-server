#[allow(unused_variables)]
use crate::common::*;
use spa_server::config::get_host_path_from_domain;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

mod common;

fn get_cert_path() -> PathBuf {
    get_test_dir().join("web/acme")
}
fn clean_cert() {
    let cert = get_cert_path();
    if cert.exists() {
        fs::remove_dir_all(&cert).unwrap();
    }
}
// Attention: must run pebble server firstly, then run it. the bash is at /tests/bash/run_pebble.sh
#[tokio::test]
async fn simple_acme_test() {
    let domain = LOCAL_HOST.to_owned() + "/27";
    let domain = &domain;
    let request_prefix = format!("https://{LOCAL_HOST}:8443/27");
    let request_prefix = &request_prefix;
    clean_web_domain_dir(LOCAL_HOST);
    clean_cert();
    let server = run_server_with_config("server_config_acme.conf");
    sleep(Duration::from_secs(2)).await;
    upload_file_and_check(domain, request_prefix, 1, vec![]).await;

    let (api, _) = get_client_api("client_config.conf");
    let mut wait_count = 0;
    loop {
        assert!(wait_count < 60, "60 seconds doest not have cert");
        sleep(Duration::from_secs(1)).await;
        let cert_info = api
            .get_acme_cert_info(Some(get_host_path_from_domain(domain).0.to_string()))
            .await
            .unwrap();
        if !cert_info.is_empty() {
            break;
        }
        wait_count += 1;
    }

    assert_files(domain, request_prefix, 1, vec!["index.html"]).await;

    assert_redirect_correct(request_prefix, "/27/").await;
    assert_redirect_correct(
        &format!("http://{LOCAL_HOST}:5002/27"),
        &format!("https://{LOCAL_HOST}:8443/27"),
    )
    .await;

    assert_files(domain, request_prefix, 1, vec![""]).await;
    assert_files(
        domain,
        &format!("http://{LOCAL_HOST}:5002/27"),
        1,
        vec!["", "index.html"],
    )
    .await;

    /*
    // why it could not stop
    wait_count = 0;
    server.abort();
    println!("begin to loop server close");
    loop {
        assert!(wait_count < 10, "20 seconds server does not stop");
        sleep(Duration::from_secs(2)).await;
        let cert_info = api.get_domain_info(Some(get_host_path_from_domain(domain).0.to_string())).await;
        if cert_info.is_err() {
            break
        }
        wait_count +=1;
    }
    // sometimes it output error. don't know why
    run_server_with_config("server_config_acme.conf");
    sleep(Duration::from_secs(2)).await;
    assert_files(domain, request_prefix, 1, vec!["", "index.html"]).await;
     */
}
#[ignore]
#[tokio::test]
async fn simple_acme_test2() {
    let domain = LOCAL_HOST.to_owned();
    let domain = &domain;
    let request_prefix = format!("https://{LOCAL_HOST}:8443");
    let request_prefix = &request_prefix;
    clean_web_domain_dir(LOCAL_HOST);
    clean_cert();
    run_server_with_config("server_config_acme.conf");
    sleep(Duration::from_secs(2)).await;
    upload_file_and_check(domain, request_prefix, 1, vec![]).await;

    let (api, _) = get_client_api("client_config.conf");
    let mut wait_count = 0;
    loop {
        assert!(wait_count < 60, "60 seconds doest not have cert");
        sleep(Duration::from_secs(1)).await;
        let cert_info = api
            .get_acme_cert_info(Some(get_host_path_from_domain(domain).0.to_string()))
            .await
            .unwrap();
        if !cert_info.is_empty() {
            break;
        }
        wait_count += 1;
    }
    assert_files(domain, request_prefix, 1, vec!["", "index.html"]).await;
}
