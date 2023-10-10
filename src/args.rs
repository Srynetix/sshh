use std::path::PathBuf;

use clap::Parser;

/// Connect to known SSH targets from your SSH config.
///
/// If you don't see anything, please add hosts declarations
/// in your SSH config (defaults to $HOME/.ssh/config).
#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// SSH configuration path.
    #[arg(short, long)]
    config_path: Option<PathBuf>,
}

impl Args {
    /// Fetch the SSH configuration path, from command-line argument or from $HOME/.ssh directory.
    pub fn get_config_path(&self) -> PathBuf {
        if let Some(p) = &self.config_path {
            p.clone()
        } else {
            let home_dir = home::home_dir().unwrap();
            home_dir.join(".ssh/config")
        }
    }
}
