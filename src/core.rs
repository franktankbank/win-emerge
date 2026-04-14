use std::{path::PathBuf, sync::LazyLock};

use platform_dirs::AppDirs;
use git2::{Repository, Direction};
use natord::compare;
use regex::Regex;

use crate::error::CoreError;

pub static APP_DIRS: LazyLock<AppDirs> = LazyLock::new(|| {
    AppDirs::new(Some(env!("CARGO_PKG_NAME")), false).unwrap()
});

pub struct ImportantDirs {
    pub vcpkg: PathBuf,
    pub recipes: PathBuf,
    pub prefix: PathBuf,
    pub sysroot: PathBuf
}

pub static IMPORTANT_DIRS: LazyLock<ImportantDirs> = LazyLock::new(|| {
    ImportantDirs { vcpkg: APP_DIRS.data_dir.join("vcpkg"), recipes: APP_DIRS.data_dir.join("recipes"), prefix: APP_DIRS.data_dir.join("opt"), sysroot: APP_DIRS.data_dir.join("vcpkg").join("installed").join("x64-mingw-static-release") }
});

pub fn utf16le_to_string(bytes: &[u8]) -> String {
    let mut u16_vec = Vec::with_capacity(bytes.len() / 2);

    for chunk in bytes.chunks_exact(2) {
        u16_vec.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }

    String::from_utf16_lossy(&u16_vec)
}

pub fn latest_version(url: &str, method: &str, pattern: &str) -> String {
    match method {
        "git" => get_latest_tag(url, pattern).expect("failed to get latest tag"),
        _ => panic!("unknown method used")
    }
}

pub fn get_latest_tag(url: &str, pattern: &str) -> Result<String, CoreError> {
    let repo = Repository::open(".")?;
    let mut remote = repo
        .find_remote(url)
        .or_else(|_| repo.remote_anonymous(url))?;

    // Connect to the remote and call the printing function for each of the
    // remote references.
    let connection = match remote.connect_auth(Direction::Fetch, None, None) {
        Ok(connection) => connection,
        Err(e) => return Err(CoreError::GitConnection(e))
    };

    // Get the list of references on the remote and print out their name next to
    // what they point to.
    let re = match Regex::new(pattern) {
        Ok(re) => re,
        Err(e) => return Err(CoreError::InvalidRegex(e))
    };

    let mut tags: Vec<String> = connection.list()?
        .iter()
        .filter_map(|r| Some(r.name()))
        .map(|n| n.strip_prefix("refs/tags/").unwrap_or(n).to_string())
        .map(|n| n.strip_suffix("^{}").unwrap_or(&n).to_string())
        .filter(|name| re.is_match(name))
        .collect();

    // Sort naturally
    tags.sort_by(|a, b| compare(a, b));
    tags.pop().ok_or_else(|| CoreError::NoMatchRegex(pattern.to_string()))
}
