use owo_colors::OwoColorize;
use std::env::current_exe;
use std::fmt::{self};
use std::fs;

use crate::constants::SELF_UPDATE_SERVER;
use crate::helpers::{normalize_url, ResultToString};
use crate::http::{download_binary, download_binary_with_loading_indicator, get_json};
use crate::log::{GlobalLogger, Logger};

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}

impl fmt::Display for Version {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Version {
    #[inline(always)]
    fn from_cargo() -> Self {
        let major = env!("CARGO_PKG_VERSION_MAJOR");
        let minor = env!("CARGO_PKG_VERSION_MINOR");
        let patch = env!("CARGO_PKG_VERSION_PATCH");

        return Version {
            major: major.parse::<i32>().unwrap_or_default(),
            minor: minor.parse::<i32>().unwrap_or_default(),
            patch: patch.parse::<i32>().unwrap_or_default(),
        };
    }

    fn from_string(version_str: &str) -> Self {
        let mut parts = version_str.split('.');

        let major = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let minor = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let patch = parts.next().unwrap_or("0").parse().unwrap_or(0);

        return Version {
            major,
            minor,
            patch,
        };
    }
}

#[inline(always)]
pub fn pkg_name() -> String {
    let name = env!("CARGO_PKG_NAME");
    return name.to_string();
}

#[inline(always)]
pub fn current_version() -> Version {
    return Version::from_cargo();
}

#[inline(always)]
fn get_update_server() -> String {
    return normalize_url(SELF_UPDATE_SERVER, "");
}

#[inline(never)]
pub async fn get_latest(
    url: &str,
    pkg: &str,
) -> Option<Version> {
    let json = get_json(url).await?;
    let row = json.get(&pkg)?;
    let version_str = row.get("version")?.as_str()?;

    return Some(Version::from_string(version_str));
}

#[inline(always)]
#[allow(unreachable_code)]
fn get_arch() -> Result<String, String> {
    #[cfg(target_arch = "x86_64")]
    return Ok(String::from("x86_64"));

    #[cfg(target_arch = "aarch64")]
    return Ok(String::from("aarch64"));

    return Err(String::from("Unsupported cpu architecture."));
}

fn get_current_bin_location() -> Result<String, String> {
    let err = String::from("Could not determine binary location");

    let exe_path = current_exe().map_err(|_| &err)?;

    return exe_path.into_os_string().into_string().map_err(|_| err);
}

fn install_binary(
    tmp_location: &str,
    bin_location: &str,
) -> Result<(), String> {
    return fs::rename(tmp_location, bin_location).map_err_to_string();
}

async fn download_latest(
    url: &str,
    pkg: &str,
    tmp: &str,
) -> Result<String, String> {
    let bin_location = get_current_bin_location()?;
    let arch = &get_arch()?;

    let download_url = format!("{url}/{arch}/{pkg}");

    if GlobalLogger::get_verbosity().is_some() {
        download_binary_with_loading_indicator(&download_url, &tmp).await?;
    } else {
        download_binary(&download_url, &tmp).await?;
    }

    install_binary(&tmp, &bin_location)?;

    Ok(bin_location)
}

fn cleanup(file: &str) {
    fs::remove_file(file).unwrap_or_default();
}

async fn download_latest_with_cleanup(
    url: &str,
    pkg: &str,
) -> Result<String, String> {
    let tmp_location = String::from("/tmp/download-ntfy-log.bin");
    let result = download_latest(url, pkg, &tmp_location).await;

    cleanup(&tmp_location); // remove trailing tmpfile whether download and install completed or not

    return result;
}

pub async fn self_update(logger: &Logger) -> Result<i32, String> {
    let installed = current_version();
    let url = get_update_server();
    let pkg = pkg_name();

    match get_latest(&url, &pkg).await {
        Some(available) if available > installed => {
            let location = download_latest_with_cleanup(&url, &pkg).await?;

            logger.success(format!(
                "upgraded {} from {} to {}",
                location.blue(),
                installed.blue(),
                available.green()
            ));

            Ok(0)
        },
        Some(_) => {
            let msg = format!("already on the latest version ({})", installed);
            logger.log(msg.green().to_string());

            Ok(0)
        },
        None => Err(String::from("Could not get latest available version")),
    }
}
