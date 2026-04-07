use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use anyhow::{Result, Error};

#[derive(Debug)]
pub struct WslHelper {
    pub installed: bool
}

pub static DISTRO_NAME: &str = "win-emerge";
pub static WSL_USER: &str = "builder";

impl WslHelper {
    pub fn new() -> Self {
        let stdout = Command::new("wsl")
            .arg("--status")
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn process")
            .stdout
            .expect("Could not capture standard output");

        let reader = BufReader::new(stdout);

        let results = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| line.contains("WSL2 is not supported with your current machine configuration."))
            .count();

        return Self {
            installed: results > 0
        }
    }

    pub fn install(&mut self) -> Result<(), Error> {
        match Command::new("wsl").args(["--install", "--no-distribution"]).output() {
            Ok(_) => {
                self.installed = true;
                Ok(())
            },
            Err(e) => Err(e.into())
        }
    }

    pub fn setup_distro(&self, vhdx_path: &str) -> Result<(), Error> {
        match Command::new("wsl").args(["--import-in-place", DISTRO_NAME, vhdx_path]).output() {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into())
        }
    }
}
