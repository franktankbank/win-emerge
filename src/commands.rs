mod init;
mod install;

pub use init::init;
pub use install::install;

#[derive(clap::Subcommand)]
pub enum Commands {
    // Initializes emerge-win
    Init {
        // force initialization
        #[arg(short, long)]
        force: bool
    },

    Install {
        package: Option<String>
    }
}
