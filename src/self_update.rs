use owo_colors::OwoColorize;
use std::env::current_exe;
use std::fmt;
use std::fs;

use crate::constants::GITHUB_REPO;
use crate::helpers::ResultToString;
use crate::http::{download_binary, download_binary_with_loading_indicator, get_json};
use crate::log::{GlobalLogger, Logger};

const TMP_DOWNLOAD_PATH: &str = "/tmp/download-ntfy-log.bin";

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Version {
    fn from_cargo() -> Self {
        let major = env!("CARGO_PKG_VERSION_MAJOR");
        let minor = env!("CARGO_PKG_VERSION_MINOR");
        let patch = env!("CARGO_PKG_VERSION_PATCH");

        Self {
            major: major.parse().unwrap_or_default(),
            minor: minor.parse().unwrap_or_default(),
            patch: patch.parse().unwrap_or_default(),
        }
    }

    fn from_string(version_str: &str) -> Self {
        let clean_version = version_str.strip_prefix('v').unwrap_or(version_str);
        let mut parts = clean_version.split('.');

        let major = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let minor = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let patch = parts.next().unwrap_or("0").parse().unwrap_or(0);

        Self { major, minor, patch }
    }
}

pub fn current_version() -> Version {
    Version::from_cargo()
}

fn github_releases_url() -> String {
    format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO)
}

fn github_download_url(tag_name: &str, arch: &str) -> String {
    format!(
        "https://github.com/{}/releases/download/{}/ntfy-log-{}",
        GITHUB_REPO, tag_name, arch
    )
}

pub async fn get_latest() -> Result<Version, String> {
    let url = github_releases_url();

    let json = get_json(&url)
        .await
        .ok_or("Failed to fetch release data from GitHub API")?;

    let tag_name = json
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid tag_name in GitHub response")?;

    Ok(Version::from_string(tag_name))
}

fn get_arch() -> Result<&'static str, String> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("x86_64"),
        "aarch64" => Ok("arm64"),
        arch => Err(format!("Unsupported CPU architecture: {}", arch)),
    }
}

fn get_current_bin_location() -> Result<String, String> {
    let exe_path = current_exe()
        .map_err(|_| "Could not determine binary location")?;

    exe_path
        .into_os_string()
        .into_string()
        .map_err(|_| "Could not convert binary path to string".to_string())
}

fn install_binary(tmp_location: &str, bin_location: &str) -> Result<(), String> {
    fs::rename(tmp_location, bin_location).map_err_to_string()
}

async fn download_latest(tag_name: &str, tmp_path: &str) -> Result<String, String> {
    let bin_location = get_current_bin_location()?;
    let arch = get_arch()?;
    let download_url = github_download_url(tag_name, arch);

    if GlobalLogger::get_verbosity().is_some() {
        download_binary_with_loading_indicator(&download_url, tmp_path).await?;
    } else {
        download_binary(&download_url, tmp_path).await?;
    }

    install_binary(tmp_path, &bin_location)?;
    Ok(bin_location)
}

fn cleanup_temp_file(file_path: &str) {
    fs::remove_file(file_path).unwrap_or_default();
}

async fn download_latest_with_cleanup(tag_name: &str) -> Result<String, String> {
    let result = download_latest(tag_name, TMP_DOWNLOAD_PATH).await;
    cleanup_temp_file(TMP_DOWNLOAD_PATH);
    result
}

pub async fn self_update(logger: &Logger) -> Result<i32, String> {
    let installed = current_version();

    match get_latest().await {
        Ok(available) if available > installed => {
            let tag_name = available.to_string();
            let location = download_latest_with_cleanup(&tag_name).await?;

            logger.success(format!(
                "Upgraded {} from {} to {}",
                location.blue(),
                installed.blue(),
                available.green()
            ));

            Ok(0)
        }
        Ok(_) => {
            logger.log(format!(
                "Already on the latest version ({})",
                installed.to_string().green()
            ));

            Ok(0)
        }
        Err(e) => Err(format!("Could not get latest available version: {}", e)),
    }
}