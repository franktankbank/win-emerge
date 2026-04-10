use std::process::{Command, Stdio, Child};
use std::io::Write;
use std::sync::{OnceLock, Mutex};
use anyhow::{Result, Error};

#[derive(Debug)]
pub struct WslHelper {
    pub installed: bool
}

pub static DISTRO_NAME: &str = "win-emerge";
pub static WSL_USER: &str = "builder";
pub static WSL_SHELL: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

pub fn setup_shell() {
    let child = Command::new("wsl")
        .args(["--distribution", DISTRO_NAME, "--user", WSL_USER])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn wsl instance");
    WSL_SHELL.get_or_init(|| Mutex::new(Some(child)));
}

pub fn wsl_write_to_stdin(cmd: String) {
    let mut guard = WSL_SHELL.get().unwrap().lock().unwrap();
    let child = guard.as_mut().unwrap();
    let stdin = child.stdin.as_mut().unwrap();
    let _ = writeln!(stdin, "{}", cmd);
}

impl WslHelper {
    pub fn new() -> Self {
        let output = Command::new("wsl")
            .arg("--status")
            .output()
            .expect("Failed to run wsl --status");

        let results = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| line.contains(
                "WSL2 is not supported with your current machine configuration."
            ))
            .count();

        return Self {
            installed: results == 0
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
