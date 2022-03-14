#![allow(dead_code)]

#[macro_use]
extern crate log;

pub mod server;

mod config;
mod domain_storage;
mod static_file_filter;
pub use server::Server;
