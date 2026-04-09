#[path = "../core.rs"]
mod core;
#[path = "../download.rs"]
mod download;
#[path = "../decompress.rs"]
mod decompress;
#[path = "../wsl.rs"]
mod wsl;
#[path = "../windows.rs"]
mod windows;

use std::{fs, process::Command};

use anyhow::{Error, Result};
use git2::build::RepoBuilder;

use core::{APP_DIRS, IMPORTANT_DIRS, get_latest_tag};
use download::DownloadResult;
use decompress::ZstdDec;
use wsl::WslHelper;
use windows::reg;

pub fn init(force: bool) -> Result<(), Error> {
    let mut wsl_helper = WslHelper::new();

    if !wsl_helper.installed {
        println!("Wsl2 is not setup. Installing...");
        match wsl_helper.install() {
            Ok(_) => {
                println!("Please reboot your computer for the changes to take effect. Afterwards, run this command again.");
                return Ok(())
            },
            Err(e) => return Err(e.into())
        }
    }

    if APP_DIRS.data_dir.exists() {
        if !force  {
            println!("emerge-win seems to already be initialized. To proceed anyway, use '--force'");
            return Ok(())
        }
    } else {
        fs::create_dir(&APP_DIRS.data_dir)?;
    }

    if IMPORTANT_DIRS.vcpkg.exists() {
        fs::remove_dir_all(&IMPORTANT_DIRS.vcpkg)?;
    }

    fs::create_dir(&IMPORTANT_DIRS.vcpkg)?;

    let vcpkg_url: &str = "https://github.com/microsoft/vcpkg.git";
    let latest_vcpkg_version: String = get_latest_tag(vcpkg_url, r"^\d{4}\.\d{2}(\.\d{2}(\.\d+)?)?(-\d+)?$").unwrap();
    let mut repo_builder = RepoBuilder::new();
    let repo_builder = repo_builder.branch(&latest_vcpkg_version);

    repo_builder.clone(vcpkg_url, &IMPORTANT_DIRS.vcpkg)?;

    let download_result = DownloadResult::new("https://github.com/franktankbank/win-emerge-files/raw/refs/heads/main/ext4.vhdx.zst", &APP_DIRS.data_dir.join("ext4.vhdx.zst").to_str().unwrap());

    if !download_result.status {
        return Err(Error::msg(download_result.message))
    } else {
        println!("{}", download_result.message);
    }

    let input_file_str = download_result.path.unwrap();
    let output_file_str = input_file_str.clone().replace(".zst", "");

    let zstd_dec = ZstdDec::new(input_file_str.as_str(), output_file_str.as_str());

    if !zstd_dec.status {
        return Err(Error::msg(zstd_dec.message))
    } else {
        println!("{}", zstd_dec.message)
    }

    fs::remove_file(input_file_str)?;

    wsl_helper.setup_distro(&output_file_str)?;

    if IMPORTANT_DIRS.recipes.exists() {
        fs::remove_dir_all(&IMPORTANT_DIRS.recipes)?;
    }

    fs::create_dir(&IMPORTANT_DIRS.recipes)?;

    let recipes_url = "https://github.com/franktankbank/win-emerge-recipes.git";
    let mut repo_builder = RepoBuilder::new();

    repo_builder.clone(recipes_url, &IMPORTANT_DIRS.recipes)?;

    match Command::new(&IMPORTANT_DIRS.vcpkg.join("bootstrap-vcpkg.bat")).arg("-disableMetrics").output() {
        Ok(_) => (),
        Err(e) => return Err(e.into())
    };

    match reg::write_initialized_flag() {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into())
    }
}
