use std::{fs, process::Command};

use git2::build::RepoBuilder;

use crate::core::{APP_DIRS, IMPORTANT_DIRS, get_latest_tag};
use crate::download::download;
use crate::decompress::decode_zstd;
use crate::wsl::WslHelper;
use crate::windows::reg;
use crate::error::InitError;

pub fn init(force: bool) -> Result<(), InitError> {
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
        fs::create_dir(&APP_DIRS.data_dir).map_err(|e| InitError::CreateDir(e))?;
    }

    if IMPORTANT_DIRS.vcpkg.exists() {
        fs::remove_dir_all(&IMPORTANT_DIRS.vcpkg).map_err(|e| InitError::RemoveDir(e))?;
    }

    fs::create_dir(&IMPORTANT_DIRS.vcpkg).map_err(|e| InitError::CreateDir(e))?;

    let vcpkg_url: &str = "https://github.com/microsoft/vcpkg.git";
    let latest_vcpkg_version: String = get_latest_tag(vcpkg_url, r"^\d{4}\.\d{2}(\.\d{2}(\.\d+)?)?(-\d+)?$").unwrap();
    let mut repo_builder = RepoBuilder::new();
    let repo_builder = repo_builder.branch(&latest_vcpkg_version);

    repo_builder.clone(vcpkg_url, &IMPORTANT_DIRS.vcpkg)?;

    let input_file_str = download("https://github.com/franktankbank/win-emerge-files/raw/refs/heads/main/ext4.vhdx.zst", &APP_DIRS.data_dir.join("ext4.vhdx.zst").to_str().unwrap())?;
    let output_file_str = input_file_str.clone().replace(".zst", "");

    decode_zstd(input_file_str.as_str(), output_file_str.as_str())?;

    fs::remove_file(input_file_str).map_err(|e| InitError::RemoveFile(e))?;

    wsl_helper.setup_distro(&output_file_str)?;

    if IMPORTANT_DIRS.recipes.exists() {
        fs::remove_dir_all(&IMPORTANT_DIRS.recipes).map_err(|e| InitError::RemoveDir(e))?;
    }

    fs::create_dir(&IMPORTANT_DIRS.recipes).map_err(|e| InitError::CreateDir(e))?;

    let recipes_url = "https://github.com/franktankbank/win-emerge-recipes.git";
    let mut repo_builder = RepoBuilder::new();

    repo_builder.clone(recipes_url, &IMPORTANT_DIRS.recipes)?;

    Command::new(&IMPORTANT_DIRS.vcpkg.join("bootstrap-vcpkg.bat")).arg("-disableMetrics").output().map_err(|e| InitError::Cmd(e))?;

    Ok(reg::write_initialized_flag().map_err(|e| InitError::Reg(e))?)
}
