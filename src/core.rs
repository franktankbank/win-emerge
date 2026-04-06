use std::{path::PathBuf, sync::LazyLock};

use platform_dirs::AppDirs;
use git2::{Repository, Direction};
use natord::compare;
use regex::Regex;
use anyhow::anyhow;

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

pub fn latest_version(url: &str, method: &str, pattern: &str) -> String {
    match method {
        "git" => get_latest_tag(url, pattern).expect("failed to get latest tag"),
        _ => panic!("unknown method used")
    }
}

pub fn get_latest_tag(url: &str, pattern: &str) -> Result<String, anyhow::Error> {
    let repo = Repository::open(".")?;
    let mut remote = repo
        .find_remote(url)
        .or_else(|_| repo.remote_anonymous(url))?;

    // Connect to the remote and call the printing function for each of the
    // remote references.
    let connection = match remote.connect_auth(Direction::Fetch, None, None) {
        Ok(connection) => connection,
        Err(e) => panic!("Failed to connect to remote {}", e)
    };

    // Get the list of references on the remote and print out their name next to
    // what they point to.
    let re = Regex::new(pattern)
        .map_err(|e| anyhow!("Invalid regex: {}", e))?;

    let mut tags: Vec<String> = connection.list()?
        .iter()
        .filter_map(|r| Some(r.name()))
        .map(|n| n.strip_prefix("refs/tags/").unwrap_or(n).to_string())
        .filter(|name| re.is_match(name))
        .collect();

    // Sort naturally
    tags.sort_by(|a, b| compare(a, b));
    tags.pop().ok_or_else(|| anyhow!("No tags matched regex: {}", pattern))
}
