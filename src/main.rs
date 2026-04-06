mod core;
mod commands;
#[path = "config/config.rs"]
mod config;

use clap::Parser;

use commands::init;
use commands::install;
use commands::Commands;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>
}

fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { force }) => {
            init(*force)
        },
        Some(Commands::Install { package }) => {
            install(package.as_ref().expect("No package given").as_str())
        }
        None => Ok(())
    }
}
