use std::path::Path;

use mlua::Lua;
use git2::build::RepoBuilder;

use crate::config::{PackageRuntime, load_package, Package};
use crate::windows::reg;
use crate::wsl::{WSL_USER, wsl_write_to_stdin};
use crate::error::{InstallError, ConfigError};

pub fn install(package: &str) -> Result<(), InstallError> {
    match reg::read_initialized_flag() {
        Ok(result) => {
            match result {
                true => (),
                false => return Err(InstallError::Init)
            }
        },
        Err(_) => return Err(InstallError::Init)
    };
    let lua: Lua = Lua::new();

    let pkg: Package = load_package(&lua, package)?;

    let mut repo_builder = RepoBuilder::new();
    repo_builder.branch(&pkg.metadata.version);
    let path = Path::new("/home").join(WSL_USER).join(&pkg.metadata.name);
    let _ = repo_builder.clone(&pkg.metadata.source_url, path.as_path())?;

    let runtime: PackageRuntime = PackageRuntime::new(&lua, pkg.metadata.build_mode.as_str(), path.clone()).map_err(ConfigError::Lua)?;

    println!("Building: {}", package);
    runtime.run_build(&pkg.build)?;

    println!("Installing: {}", package);
    runtime.run_install(&pkg.install)?;

    wsl_write_to_stdin("cd".to_string())?;
    wsl_write_to_stdin(format!("rm -rf '{}'", path.to_str().unwrap()))?;

    Ok(())
}
