use std::process::{Command, Stdio, Child};
use std::io::Write;
use std::sync::{OnceLock, Mutex};

use crate::core::utf16le_to_string;
use crate::error::WslError;
use crate::windows::elevate;

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
        .output();

        if let Ok(output) = output {
            let stdout = utf16le_to_string(&output.stdout).to_lowercase();
            return Ok(Self {
                installed: stdout.contains("default version: 2") || stdout.contains("wsl version: 2")
            })
        } else {
            return Ok(Self {
                installed: false
            })
        }
    }

    pub fn install(&mut self) -> Result<(), WslError> {
        elevate::elevate_if_needed()?;
        let code = Command::new("powershell").args(["-NoProfile", "-Command", "wsl --install --no-distribution"]).spawn()?.wait()?;
        if code.success() {
            self.installed = true;
            Ok(())
        } else {
            Err(WslError::Child)
        }
    }

    pub fn setup_distro(&self, vhdx_path: &str) -> Result<(), WslError> {
        let code = Command::new("powershell").args(["-NoProfile", "-Command", format!("wsl --import-in-place {} {}", DISTRO_NAME, vhdx_path).as_str()]).spawn()?.wait()?;
        if code.success() {
            Ok(())
        } else {
            Err(WslError::Child)
        }
    }
}
