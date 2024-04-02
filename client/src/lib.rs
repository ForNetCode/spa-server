#![allow(dead_code)]
#![allow(unused_variables)]

mod api;
pub mod commands;
mod config;
mod upload_files;

use crate::api::API;
use crate::commands::{CliCommand, Commands};
use crate::config::Config;
use crate::upload_files::upload_files;
use anyhow::anyhow;

use clap::Parser;
use console::style;

// this is for bin
pub fn run() {
    let commands = CliCommand::parse();
    let result = run_with_commands(commands);
    if let Some(err) = result.err() {
        eprintln!("{}", err);
        std::process::exit(-1);
    }
}
//this is for js
pub fn run_js() {
    let args = std::env::args_os().skip(1);
    // println!("{:?}", &args);
    let commands = CliCommand::parse_from(args);
    let result = run_with_commands(commands);
    if let Some(err) = result.err() {
        eprintln!("{}\n", err);
        std::process::exit(1);
    }
}

fn success(message: &str) {
    println!("{}", style(message).green());
}

fn run_with_commands(commands: CliCommand) -> anyhow::Result<()> {
    let config = Config::load(commands.config_dir).map_err(|e| {
        anyhow!(
            "Please set config file path or environment variable correctly, {}",
            e
        )
    })?;
    println!(
        "spa-client connect to admin server({})",
        &config.server.address
    );
    let api = API::new(&config)?;

    match commands.commands {
        Commands::Info { domain } => {
            println!("{}", api.get_domain_info(domain)?);
        }
        Commands::Upload(arg) => {
            let parallel = arg.parallel.unwrap_or(config.upload.parallel);
            upload_files(api, arg.domain, arg.version, arg.path, parallel)?;
        }
        Commands::Release { domain, version } => {
            let resp = api.release_domain_version(domain, version)?;
            success(&resp);
        }
        Commands::Reload => {
            api.reload_spa_server()?;
            success("reload success!");
        }
        Commands::Delete {
            domain,
            max_reserve,
        } => {
            api.remove_files(domain, max_reserve)?;
            success("delete success!");
        }
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{run, run_with_commands, CliCommand};
    use clap::Parser;
    use std::env;

    fn init_config() {
        env::set_var("SPA_SERVER_ADDRESS", "http://127.0.0.1:9000");
        env::set_var("SPA_SERVER_AUTH_TOKEN", "token");
        env::set_var("SPA_UPLOAD_PARALLEL", "4");
    }

    #[test]
    fn test_info() {
        init_config();
        run_with_commands(CliCommand::parse_from(&["test", "info"])).unwrap();
    }

    #[test]
    fn test_upload() {
        init_config();
        let ret = run_with_commands(CliCommand::parse_from(&[
            "test",
            "upload",
            "../example/js-app-example/build",
            "self.noti.link",
        ]));

        if let Err(ret) = ret {
            println!("{:?}", ret);
        }
    }
    #[test]
    fn test_release() {
        init_config();
        let result = run_with_commands(CliCommand::parse_from(&[
            "test",
            "release",
            "self.noti.link",
        ]));
        result.unwrap();
    }
    #[test]
    fn test_delete() {
        init_config();
        let result = run_with_commands(CliCommand::parse_from(&[
            "test",
            "delete",
            "self.noti.link",
            "2",
        ]));
        result.unwrap();
    }
}
