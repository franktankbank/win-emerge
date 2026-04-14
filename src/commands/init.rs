use std::{fs, process::Command};

use git2::Repository;
use deelevate::spawn_with_elevated_privileges;

use crate::core::{APP_DIRS, IMPORTANT_DIRS, get_latest_tag};
use crate::download::download;
use crate::decompress::decode_zstd;
use crate::wsl::WslHelper;
use crate::windows::reg;
use crate::error::InitError;

pub fn init(force: bool) -> Result<(), InitError> {
    spawn_with_elevated_privileges();
    let mut wsl_helper = WslHelper::new()?;

    if !wsl_helper.installed {
        println!("Wsl2 is not setup. Installing...");
        wsl_helper.install()?;
        println!("Please reboot your computer for the changes to take effect. Afterwards, run this command again.");
        return Ok(())
    }

    if APP_DIRS.data_dir.exists() {
        if !force  {
            println!("emerge-win seems to already be initialized. To proceed anyway, use '--force'");
            return Ok(())
        }
    } else {
        println!("Creating data directory: {}", APP_DIRS.data_dir.display());
        fs::create_dir(&APP_DIRS.data_dir).map_err(|e| InitError::CreateDir(e))?;
    }

    if IMPORTANT_DIRS.vcpkg.exists() {
        fs::remove_dir_all(&IMPORTANT_DIRS.vcpkg).map_err(|e| InitError::RemoveDir(e))?;
    }

    println!("Creating vcpkg directory: {}", &IMPORTANT_DIRS.vcpkg.display());
    fs::create_dir(&IMPORTANT_DIRS.vcpkg).map_err(|e| InitError::CreateDir(e))?;

    let vcpkg_url: &str = "https://github.com/microsoft/vcpkg.git";
    println!("Finding latest vcpkg version");
    let latest_vcpkg_version: String = get_latest_tag(vcpkg_url, r"^\d{4}\.\d{2}(\.\d{2}(\.\d+)?)?(-\d+)?$").unwrap();

    println!("Fetching vcpkg");
    let repo = Repository::clone(vcpkg_url, &IMPORTANT_DIRS.vcpkg)?;

    let (object, reference) = repo.revparse_ext(&latest_vcpkg_version)?;
    repo.checkout_tree(&object, None)?;
    match reference {
        Some(gref) => repo.set_head(gref.name().unwrap())?,
        None => repo.set_head_detached(object.id())?
    }

    println!("downloading wsl image");
    let input_file_str = download("https://github.com/franktankbank/win-emerge-files/raw/refs/heads/main/ext4.vhdx.zst", &APP_DIRS.data_dir.join("ext4.vhdx.zst").to_str().unwrap())?;
    let output_file_str = input_file_str.clone().replace(".zst", "");

    println!("Decompressing wsl image");
    decode_zstd(input_file_str.as_str(), output_file_str.as_str())?;

    fs::remove_file(input_file_str).map_err(|e| InitError::RemoveFile(e))?;

    println!("setting up distro from wsl image");
    wsl_helper.setup_distro(&output_file_str)?;

    if IMPORTANT_DIRS.recipes.exists() {
        fs::remove_dir_all(&IMPORTANT_DIRS.recipes).map_err(|e| InitError::RemoveDir(e))?;
    }

    println!("Creating recipe directory: {}", &IMPORTANT_DIRS.recipes.display());
    fs::create_dir(&IMPORTANT_DIRS.recipes).map_err(|e| InitError::CreateDir(e))?;

    let recipes_url = "https://github.com/franktankbank/win-emerge-recipes.git";

    println!("Cloning recipes");
    Repository::clone(recipes_url, &IMPORTANT_DIRS.recipes)?;

    println!("Installing vcpkg");
    Command::new(&IMPORTANT_DIRS.vcpkg.join("bootstrap-vcpkg.bat")).arg("-disableMetrics").output().map_err(|e| InitError::Cmd(e))?;

    println!("Initialization complete");
    Ok(reg::write_initialized_flag().map_err(|e| InitError::Reg(e))?)
}
