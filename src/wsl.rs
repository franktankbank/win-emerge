use std::process::{Command, Stdio, Child};
use std::io::Write;
use std::sync::{OnceLock, Mutex};

use crate::error::WslError;

#[derive(Debug)]
pub struct WslHelper {
    pub installed: bool
}

pub static DISTRO_NAME: &str = "win-emerge";
pub static WSL_USER: &str = "builder";
pub static WSL_SHELL: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

pub fn setup_shell() -> Result<(), WslError> {
    let child = Command::new("wsl")
        .args(["--distribution", DISTRO_NAME, "--user", WSL_USER])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    WSL_SHELL.get_or_init(|| Mutex::new(Some(child)));

    Ok(())
}

pub fn wsl_write_to_stdin(cmd: String) -> Result<(), WslError> {
    let mut guard = match WSL_SHELL.get() {
        Some(mutex) => mutex,
        None => return Err(WslError::Mutex(stringify!(WSL_SHELL).to_string()))
    }.lock()?;
    let child = match guard.as_mut() {
        Some(child) => child,
        None => return Err(WslError::Child)
    };
    let stdin = match child.stdin.as_mut() {
        Some(stdin) => stdin,
        None => return Err(WslError::Stdin)
    };
    let _ = writeln!(stdin, "{}", cmd);

    Ok(())
}

impl WslHelper {
    pub fn new() -> Result<Self, WslError> {
        let output = Command::new("wsl")
            .arg("--status")
            .output()?;

        let results = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| line.contains(
                "WSL2 is not supported with your current machine configuration."
            ))
            .count();

        return Ok(Self {
            installed: results == 0
        })
    }

    pub fn install(&mut self) -> Result<(), WslError> {
        match Command::new("wsl").args(["--install", "--no-distribution"]).output() {
            Ok(_) => {
                self.installed = true;
                Ok(())
            },
            Err(e) => Err(WslError::Command(e))
        }
    }

    pub fn setup_distro(&self, vhdx_path: &str) -> Result<(), WslError> {
        match Command::new("wsl").args(["--import-in-place", DISTRO_NAME, vhdx_path]).output() {
            Ok(_) => Ok(()),
            Err(e) => Err(WslError::Command(e))
        }
    }
}
