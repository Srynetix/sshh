//! SSH config parser.

use std::path::Path;

use crate::config::{SshConfigFile, SshConfigHost};

#[derive(Default)]
pub struct SshConfigParser;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Could not read file: {0}")]
    CouldNotReadFile(String),
}

impl SshConfigParser {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn parse_from_path<P: AsRef<Path>>(&self, path: P) -> Result<SshConfigFile, ParseError> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ParseError::CouldNotReadFile(e.to_string()))?;
        self.parse(&contents)
    }

    pub fn parse(&self, value: &str) -> Result<SshConfigFile, ParseError> {
        let mut config_file = SshConfigFile::default();

        let mut current_host_alias: Option<&str> = None;
        let mut current_host_definition: Option<SshConfigHost> = None;

        for line in value.lines() {
            let line = line.trim();

            if line.starts_with('#') || line.is_empty() {
                // That's a comment, let's skip
                continue;
            }

            // Let's eat words
            let words = line.split_ascii_whitespace().collect::<Vec<_>>();
            if words.len() == 2 {
                let kw = words[0];
                let value = words[1];

                if let Some(def) = current_host_definition.as_mut() {
                    match kw {
                        "Host" => {
                            config_file.hosts.insert(
                                current_host_alias.unwrap().to_owned(),
                                current_host_definition.take().unwrap(),
                            );

                            current_host_alias = Some(value);
                            current_host_definition = Some(SshConfigHost {
                                hostname: "".into(),
                                ..Default::default()
                            })
                        }
                        "HostName" => {
                            def.hostname = value.into();
                        }
                        "User" => {
                            def.user = Some(value.into());
                        }
                        "Port" => {
                            def.port = value.parse().ok();
                        }
                        "IdentityFile" => {
                            def.identity_file = Some(value.into());
                        }
                        _ => (),
                    }
                } else if kw == "Host" {
                    current_host_alias = Some(value);
                    current_host_definition = Some(SshConfigHost {
                        hostname: "".into(),
                        ..Default::default()
                    })
                }
            }
        }

        if current_host_definition.is_some() {
            config_file.hosts.insert(
                current_host_alias.unwrap().to_owned(),
                current_host_definition.take().unwrap(),
            );
        }

        Ok(config_file)
    }
}
