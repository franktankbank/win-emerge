#[path = "../config/config.rs"]
mod config;

use mlua::Lua;

use config::{PackageRuntime, load_package, Package};

pub fn install(package: &str) -> anyhow::Result<()> {
    let lua: Lua = Lua::new();

    let pkg: Package = load_package(&lua, package)?;

    let runtime: PackageRuntime = PackageRuntime::new(&lua)?;

    println!("Building: {}", package);
    runtime.run_build(&pkg.build)?;

    println!("Installing: {}", package);
    runtime.run_install(&pkg.install)?;

    Ok(())
}
