#[path = "../config/config.rs"]
mod config;
#[path = "../windows.rs"]
mod windows;
#[path = "../wsl.rs"]
mod wsl;

use std::path::Path;

use mlua::Lua;
use anyhow::{Result, Error};
use git2::build::RepoBuilder;

use config::{PackageRuntime, load_package, Package};
use windows::reg;
use wsl::WSL_USER;

use crate::commands::install::wsl::wsl_write_to_stdin;

pub fn install(package: &str) -> Result<()> {
    match reg::read_initialized_flag() {
        true => (),
        false => return Err(Error::msg("win-emerge is not initialized. Please run 'win-emerge init'"))
    };
    let lua: Lua = Lua::new();

    let pkg: Package = load_package(&lua, package)?;

    let mut repo_builder = RepoBuilder::new();
    repo_builder.branch(&pkg.metadata.version);
    let path = Path::new("/home").join(WSL_USER).join(&pkg.metadata.name);
    let _ = repo_builder.clone(&pkg.metadata.source_url, path.as_path())?;

    let runtime: PackageRuntime = PackageRuntime::new(&lua, &pkg.metadata.build_mode.as_str(), path.clone())?;

    println!("Building: {}", package);
    runtime.run_build(&pkg.build)?;

    println!("Installing: {}", package);
    runtime.run_install(&pkg.install)?;

    wsl_write_to_stdin("cd".to_string());
    wsl_write_to_stdin(format!("rm -rf '{}'", path.to_str().unwrap()));

    Ok(())
}
