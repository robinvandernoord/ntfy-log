use owo_colors::OwoColorize;

use serde_json::Value;
use std::fs::File;
use std::io::{self, copy, Write};
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;
use tokio::task;

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

// async fn fake_download_binary(_: &str, _: &str) -> Result<(), String> {
//     // Simulating download
//     tokio::time::sleep(Duration::from_secs(5)).await;
//     Ok(())
// }

pub async fn download_binary_with_loading_indicator(
    download_url: &str,
    bin_location: &str,
) -> Result<(), String> {
    let spinner = task::spawn(async {
        let spinner_chars = vec!['|', '/', '-', '\\'];
        let mut idx = 0;
        loop {
            eprint!("\rDownloading {} ", spinner_chars[idx]);
            idx = (idx + 1) % spinner_chars.len();
            io::stdout().flush().unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    let download_task = download_binary(download_url, bin_location);

    let download_result = download_task.await;
    spinner.abort(); // Abort the spinner loop as download completes
    eprint!("\r\x1B[2K"); // clear the line

    return download_result;
}
