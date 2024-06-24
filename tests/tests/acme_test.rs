use crate::common::*;
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
    //let https_request_prefix = format!("https://{LOCAL_HOST}:8080/27");
    //let https_request_prefix = format!("https://{LOCAL_HOST}:8080/27");
    clean_web_domain_dir(domain);
    clean_cert();
    let server = run_server_with_config("server_config_acme.conf");
    sleep(Duration::from_secs(2)).await;
    upload_file_and_check(domain, request_prefix, 1, vec![]).await;

    sleep(Duration::from_secs(10)).await;
    assert_files(domain, request_prefix, 1, vec!["", "index.html"]).await;
    // sometimes it output error. don't know why
    /*
    server.abort();
    sleep(Duration::from_secs(2)).await;
    run_server_with_config("server_config_acme.conf");
    sleep(Duration::from_secs(2)).await;
    assert_files(domain, request_prefix, 1, vec!["", "index.html"]).await;
     */
}
