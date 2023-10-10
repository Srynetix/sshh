use sshh::args::Args;

use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config_path = args.get_config_path();
    sshh::ui::run(config_path)
}
