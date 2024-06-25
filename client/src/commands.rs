use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version, author)]
pub struct CliCommand {
    #[clap(subcommand)]
    pub commands: Commands,
    #[clap(long, short, parse(from_os_str), value_name = "SPA_CLIENT_CONFIG")]
    pub config_dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Info {
        domain: Option<String>,
    },
    Upload(UploadArg),
    Release {
        domain: String,
        version: Option<u32>,
    },
    Reload,
    Delete {
        domain: Option<String>,
        max_reserve: Option<u32>,
    },
    Revoke {
        domain: String,
        version: u32,
    }
}

#[derive(Args, Debug)]
pub struct UploadArg {
    #[clap(parse(from_os_str))]
    pub path: PathBuf,
    pub domain: String,
    pub version: Option<u32>,
    #[clap(short)]
    pub parallel: Option<u32>,
}

#[cfg(test)]
mod test {
    use crate::commands::{CliCommand, Commands, UploadArg};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn info_command_test() {
        let c = CliCommand::try_parse_from(&["spa-client", "info", "www.example.com"]).unwrap();
        //println!("{:?}", c);
        if let Commands::Info { domain } = c.commands {
            assert_eq!(domain, Some("www.example.com".to_string()));
        } else {
            unreachable!();
        };
    }
    #[test]
    fn info_command_with_config_dir() {
        let c = CliCommand::parse_from(&["spa-client", "-c=abc.conf", "info"]);
        //println!("{:?}", &c);
        assert_eq!(c.config_dir, Some(PathBuf::from("abc.conf")));
        let c = CliCommand::parse_from(&["spa-client", "--config-dir=abc.conf", "info"]);
        //println!("{:?}", &c);
        assert_eq!(c.config_dir, Some(PathBuf::from("abc.conf")));
        let c = CliCommand::parse_from(&["spa-client", "info", "www.example.com"]);
        if let Commands::Info { domain } = c.commands {
            assert_eq!(domain.unwrap(), "www.example.com")
        } else {
            unreachable!()
        }
    }

    #[test]
    fn upload_command() {
        let c = CliCommand::parse_from(&[
            "test",
            "upload",
            "/abc/d/",
            "www.example.com",
            "2",
            "-p",
            "2",
        ]);
        if let Commands::Upload(UploadArg {
            path,
            domain,
            version,
            parallel,
        }) = c.commands
        {
            assert_eq!(path, PathBuf::from("/abc/d"));
            assert_eq!(parallel, Some(2));
            assert_eq!(domain, "www.example.com".to_string());
            assert_eq!(version, Some(2));
        } else {
            unreachable!()
        }
    }
    #[test]
    fn release_command() {
        let c = CliCommand::parse_from(&["test", "release", "www.example.com", "2"]);
        // println!("{:?}", &c);
        if let Commands::Release { domain, version } = c.commands {
            assert_eq!(domain, "www.example.com".to_string());
            assert_eq!(version, Some(2));
        } else {
            unreachable!()
        }
    }
    #[test]
    fn release_command2() {
        let c = CliCommand::parse_from(&["test", "release", "www.example.com"]);
        if let Commands::Release { domain, version } = c.commands {
            assert_eq!(domain, "www.example.com".to_string());
            assert_eq!(version, None);
        } else {
            unreachable!()
        }
    }

    #[test]
    fn revoke_version() {
        let c = CliCommand::parse_from(&["test", "revoke", "www.example.com", "1"]);
        if let Commands::Revoke {domain, version} = c.commands {
            assert_eq!(domain, "www.example.com".to_string());
            assert_eq!(version, 1);
        } else {
            unreachable!()
        }
    }
}
