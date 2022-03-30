use napi_derive::napi;

#[napi]
pub fn run_js() {
    spa_client::run_js();
}
