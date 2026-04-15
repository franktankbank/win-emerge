use std::{io, sync::PoisonError};

use thiserror::Error;
use indicatif::style::TemplateError;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error(transparent)]
    ProgressBarSetup(#[from] TemplateError),
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Couldn't determine file size.")]
    FileSize,
    #[error("Failed to create file: {0}")]
    FileCreation(io::Error),
    #[error("Read error: {0}")]
    FileRead(io::Error),
    #[error("Write error: {0}")]
    FileWrite(io::Error)
}

#[derive(Error, Debug)]
pub enum WslError {
    #[error("failed to get mutex from static var `{0}`")]
    Mutex(String),
    #[error("Lock poisoned")]
    LockPoisoned,
    #[error("Failed to get child from guard")]
    Child,
    #[error("Failed to get stdin from child")]
    Stdin,
    #[error(transparent)]
    Command(#[from] io::Error)
}

impl<T> From<PoisonError<T>> for WslError {
    fn from(_: PoisonError<T>) -> Self {
        WslError::LockPoisoned
    }
}

#[derive(Error, Debug)]
pub enum ZstdError {
    #[error("Zstd decode failed: {0}")]
    Dec(#[from] io::Error)
}

#[derive(Error, Debug)]
pub enum CoreError {
    #[error(transparent)]
    GitGeneric(#[from] git2::Error),
    #[error("Failed to connect to remote {0}")]
    GitConnection(git2::Error),
    #[error("Invalid regex: {0}")]
    InvalidRegex(regex::Error),
    #[error("No tags matched regex: '{0}'")]
    NoMatchRegex(String)
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    Lua(#[from] mlua::Error),
    #[error(transparent)]
    FileRead(#[from] io::Error),
    #[error("Lua recipe file did not call package")]
    PackageNotCalled,
    #[error("Failed to get first char of package name")]
    FirstChar
}

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("win-emerge is not initialized. Please run 'win-emerge init'")]
    Init,
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Git(#[from] git2::Error),
    #[error(transparent)]
    Lua(#[from] mlua::Error),
    #[error(transparent)]
    Wsl(#[from] WslError)
}

#[derive(Error, Debug)]
pub enum InitError {
    #[error(transparent)]
    Wsl(#[from] WslError),
    #[error(transparent)]
    CreateDir(io::Error),
    #[error(transparent)]
    RemoveDir(io::Error),
    #[error(transparent)]
    RemoveFile(io::Error),
    #[error(transparent)]
    Reg(io::Error),
    #[error(transparent)]
    Cmd(io::Error),
    #[error(transparent)]
    Git(#[from] git2::Error),
    #[error(transparent)]
    Download(#[from] DownloadError),
    #[error(transparent)]
    Zstd(#[from] ZstdError)
}

#[derive(Error, Debug)]
pub enum WinEmergeError {
    #[error(transparent)]
    Download(#[from] DownloadError),
    #[error(transparent)]
    Wsl(#[from] WslError),
    #[error(transparent)]
    Zstd(#[from] ZstdError),
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Install(#[from] InstallError),
    #[error(transparent)]
    Init(#[from] InitError),
    #[error("No package given")]
    NoPackageGiven
}
