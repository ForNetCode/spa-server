use std::time::Duration;
mod common;
use crate::common::{assert_files, assert_files_no_exists, clean_test_dir, upload_file_and_check};
use common::run_server;

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn start_server_and_client_upload_file() {
    let domain = "self.noti.link/27";
    let request_prefix = "http://self.noti.link:8080/27";

    clean_test_dir("self.noti.link");

    run_server();

    tokio::time::sleep(Duration::from_secs(2)).await;

    upload_file_and_check(domain, request_prefix, 1, vec!["index.html"]).await;

    upload_file_and_check(domain, request_prefix, 2, vec!["index.html", "2.html"]).await;

    assert_files(domain, request_prefix, 1, vec!["1.html"]).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn multiple_domain_check() {
    clean_test_dir("self.noti.link");

    let domain = "self.noti.link/27";
    let request_prefix = "http://self.noti.link:8080/27";
    let domain2 = "self.noti.link/a";
    let request_prefix2 = "http://self.noti.link:8080/a";

    run_server();

    tokio::time::sleep(Duration::from_secs(1)).await;

    upload_file_and_check(domain, request_prefix, 1, vec!["index.html"]).await;

    upload_file_and_check(domain2, request_prefix2, 1, vec!["index.html"]).await;
    println!("finish");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn evoke_cache_when_serving_new_index() {
    // this does not work correctly.
    clean_test_dir("self.noti.link");
    let domain = "self.noti.link/27";
    let request_prefix = "http://self.noti.link:8080/27";

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
}

// TODO: to fix it
// This must run after evoke_cache_when_serving_new_index
#[tokio::test(flavor = "multi_thread", worker_threads = 3)]
async fn cool_start_server_and_serving_files() {
    let domain = "self.noti.link/27";
    let request_prefix = "http://self.noti.link:8080/27";
    run_server();
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_files(domain, request_prefix, 4, vec!["index.html", "4.html"]).await;
    assert_files(domain, request_prefix, 2, vec!["2.html"]).await;
    assert_files_no_exists(request_prefix, vec!["1.html"]).await;
}
