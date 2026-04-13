mod core;
mod commands;
mod download;
mod decompress;
mod windows;
mod wsl;
mod error;
#[path = "config/config.rs"]
mod config;

use clap::Parser;

use commands::init;
use commands::install;
use commands::Commands;
use error::WinEmergeError;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>
}

fn main() -> Result<(), WinEmergeError> {
    let cli: Cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { force }) => {
            Ok(init(*force)?)
        },
        Some(Commands::Install { package }) => {
            let pkg = match package {
                Some(pkg) => pkg.as_str(),
                None => return Err(WinEmergeError::NoPackageGiven)
            };
            Ok(install(pkg)?)
        }
        None => Ok(())
    }
}
