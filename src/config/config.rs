use std::{fs, path::PathBuf, process::Command};

use mlua::{Lua, prelude::{LuaUserData, LuaUserDataMethods, LuaError, LuaFunction, LuaAnyUserData, LuaTable}};
use num_cpus;

use crate::core::{latest_version, IMPORTANT_DIRS};
use crate::wsl::{DISTRO_NAME, setup_shell, wsl_write_to_stdin};
use crate::error::ConfigError;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Context {
    pub prefix: String,
    pub target: String,
    pub jobs: usize,
    pub mode: String
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct Vcpkg {
    pub prefix: String
}

impl LuaUserData for Vcpkg {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field("prefix", IMPORTANT_DIRS.sysroot.to_string_lossy());
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("clean", |_, _this, ()| {
            println!("Cleaning Vcpkg packages");

            let vcpkg_installed = &IMPORTANT_DIRS.vcpkg.join("installed");
            if vcpkg_installed.exists() {
                fs::remove_dir_all(vcpkg_installed).expect("Failed to clean vcpkg packages");
            }

            Ok(())
        });

        methods.add_method("foreach_dep", |_, _this, deps: Vec<String>| {
            let vcpkg_exe = &IMPORTANT_DIRS.vcpkg.join("vcpkg.exe");
            for dep in deps {
                let output = Command::new(vcpkg_exe)
                    .args(vec!["install".to_string(), dep.clone(), "--triplet=x64-mingw-static-release".to_string()])
                    .status()
                    .map_err(LuaError::external)?;

                if !output.success() {
                    return Err(LuaError::external(format!(
                        "Vcpkg failed to install {}: aborting",
                        dep
                    )));
                }
            }
            Ok(())
        });
    }
}

impl LuaUserData for Context {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("run", |_, this, cmd: String| {
            println!("Running: {}", &cmd);
            match this.mode.as_str() {
                "native" => {
                    // Split command into program + args
                    let mut parts = cmd.split_whitespace();
                    let program = parts.next().ok_or_else(|| LuaError::external("empty command"))?;

                    let output = Command::new(program)
                        .args(parts)
                        .status()
                        .map_err(LuaError::external)?;

                    if !output.success() {
                        return Err(LuaError::external(format!(
                            "command failed: {}",
                            cmd
                        )));
                    }
                },
                "wsl" => {
                        wsl_write_to_stdin(cmd).map_err(|e| LuaError::external(e))?;
                },
                _ => return Err(LuaError::external(format!(
                    "Mode '{}' is not a valid mode",
                    this.mode
                )))
            }
            Ok(())
        });
    }
}

impl Context {
    pub fn new(prefix: String, target: String, jobs: usize, mode: String) -> Self {
        Self {
            prefix: prefix,
            target: target,
            jobs: jobs,
            mode: mode
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub source_url: String,
    pub dependencies: Vec<String>,
    pub build_mode: String
}

#[derive(Clone)]
pub struct Package {
    pub metadata: PackageMetadata,
    pub build: LuaFunction,
    pub install: LuaFunction
}

pub struct PackageRuntime {
    ctx_lua: LuaAnyUserData
}

impl PackageRuntime {
    pub fn new(lua: &Lua, mode: &str, build_path: PathBuf) -> mlua::Result<Self> {
        let prefix: String = match mode {
            "native" => IMPORTANT_DIRS.prefix.to_string_lossy().into(),
            "wsl" => {
                setup_shell().map_err(|e| LuaError::external(e))?;
                wsl_write_to_stdin(format!("cd '{}'", build_path.to_str().unwrap())).map_err(|e| LuaError::external(e))?;
                wslpath2::convert(IMPORTANT_DIRS.prefix.to_str().unwrap(), Some(DISTRO_NAME), wslpath2::Conversion::WindowsToWsl, true).unwrap()
            },
            _ => return Err(LuaError::external("Invalid build_mode detected. Only 'wsl' and 'native' are valid."))
        };
        let ctx = Context::new(prefix, "x86_64-w64-mingw32".to_string(), num_cpus::get(), mode.to_string());

        let ctx_lua = lua.create_userdata(ctx)?;

        Ok(Self { ctx_lua })
    }

    pub fn run_build(&self, func: &LuaFunction) -> mlua::Result<()> {
        func.call(self.ctx_lua.clone())
    }

    pub fn run_install(&self, func: &LuaFunction) -> mlua::Result<()> {
        func.call(self.ctx_lua.clone())
    }
}

pub fn load_package(lua: &Lua, package: &str) -> Result<Package, ConfigError> {
    let globals = lua.globals();

    // Inject "latest_version" into Lua for expressions inside the recipe
    lua.globals().set("latest_version", lua.create_function(|_, (url, method, pattern): (String, String, String) | {
        Ok(latest_version(url.as_str(), method.as_str(), pattern.as_str()))
    })?)?;

    let vcpkg = Vcpkg::default();

    lua.globals().set("vcpkg", vcpkg)?;

    let package_out = std::rc::Rc::new(std::cell::RefCell::new(None::<Package>));
    let package_out_clone = package_out.clone();

    // Provide the package() function to Lua
    let package_fn = lua.create_function(move |_, table: LuaTable| {
        let name: String = table.get("name")?;

        let source: LuaTable = table.get("source")?;

        let version: String = table.get("version")?;
        let url: String = source.get("url")?;

        let mode: String = table.get("build_mode")?;

        // Extract dependencies
        let deps: Vec<String> = table.get("dependencies").unwrap_or_else(|_| Vec::new());

        // Extract build/install functions
        let build_fn: LuaFunction = table.get("build")
            .map_err(|_| LuaError::external("missing build function"))?;
        let install_fn: LuaFunction = table.get("install")
            .map_err(|_| LuaError::external("missing install function"))?;

        let metadata: PackageMetadata = PackageMetadata {
            name,
            version,
            source_url: url,
            dependencies: deps,
            build_mode: mode
        };
        // Store everything in Package struct
        let pkg = Package {
            metadata,
            build: build_fn,
            install: install_fn
        };

        *package_out_clone.borrow_mut() = Some(pkg);

        Ok(())
    })?;


    globals.set("package", package_fn)?;

    // Load and run the Lua file
    let path = &IMPORTANT_DIRS.recipes.join(format!("{}.lua", package));
    let code = fs::read_to_string(path)?;
    lua.load(&code).exec()?;

    // Retrieve the constructed package
    let pkg = match package_out.borrow_mut().take() {
        Some(pkg) => pkg,
        None => return Err(ConfigError::PackageNotCalled)
    };

    Ok(pkg)

}
