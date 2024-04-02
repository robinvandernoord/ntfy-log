use owo_colors::OwoColorize;

use serde_json::Value;
use std::fs::File;
use std::io::copy;
use std::os::unix::fs::PermissionsExt;

pub async fn get_json(url: &str) -> Option<Value> {
    let response = reqwest::get(url).await;
    return match response {
        Ok(response) => {
            let json: Value = response.json().await.ok()?;
            return Some(json);
        }
        Err(e) => {
            eprintln!(">> ntfy | {} | {}", "error".red(), e.to_string());
            None
        }
    };
}

pub async fn download_binary(download_url: &str, bin_location: &str) -> Result<(), String> {
    // Send a GET request to the download URL
    let response = reqwest::get(download_url)
        .await
        .map_err(|e| e.to_string())?;

    // Ensure the request was successful (status code 200)
    if !response.status().is_success() {
        return Err(format!("Failed to download binary: {}", response.status()));
    }

    // Create a new file at the specified location
    let mut file = File::create(bin_location).map_err(|e| e.to_string())?;

    // Copy the response body (binary data) to the file
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    copy(&mut bytes.as_ref(), &mut file).map_err(|e| e.to_string())?;

    // set perms:
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    let mut permissions = metadata.permissions();

    // 755 = rwx, rx, rx
    permissions.set_mode(0o755);
    file.set_permissions(permissions)
        .map_err(|e| e.to_string())?;

    Ok(())
}
