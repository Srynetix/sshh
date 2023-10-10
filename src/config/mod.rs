pub mod parser;

use std::{collections::HashMap, path::PathBuf};

use itertools::Itertools;

#[derive(Debug, Default)]
pub struct SshConfigFile {
    pub hosts: HashMap<String, SshConfigHost>,
}

#[derive(Debug, Default)]
pub struct SshConfigHost {
    pub hostname: String,
    pub user: Option<String>,
    pub identity_file: Option<PathBuf>,
    pub port: Option<u16>,
}

#[derive(Debug, Default, Clone)]
pub struct SshHost {
    pub alias: String,
    pub hostname: String,
    pub user: Option<String>,
    pub identity_file: Option<PathBuf>,
    pub port: Option<u16>,
}

impl SshHost {
    pub fn get_command_line(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(i) = &self.identity_file {
            args.push("-i".to_string());
            args.push(i.to_string_lossy().to_string());
        }

        if let Some(p) = &self.port {
            args.push("-p".to_string());
            args.push(p.to_string());
        }

        let mut address = String::new();
        if let Some(u) = &self.user {
            address.push_str(u);
            address.push('@');
        }
        address.push_str(&self.hostname);
        args.push(address);

        args
    }
}

impl SshConfigFile {
    pub fn to_hosts(self) -> Vec<SshHost> {
        self.hosts
            .into_iter()
            .sorted_by_key(|(key, _)| key.clone())
            .map(|(key, host)| SshHost {
                alias: key,
                hostname: host.hostname,
                identity_file: host.identity_file,
                port: host.port,
                user: host.user,
            })
            .collect()
    }
}
