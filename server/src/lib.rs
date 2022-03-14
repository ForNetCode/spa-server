#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate log;

pub mod server;

mod admin_server;
mod config;
mod domain_storage;
mod static_file_filter;

// utils
pub use server::Server;
use std::convert::Infallible;
use std::sync::Arc;
use warp::Filter;

pub fn with<T: Send + Sync>(
    d: Arc<T>,
) -> impl Filter<Extract = (Arc<T>,), Error = Infallible> + Clone {
    warp::any().map(move || d.clone())
}
