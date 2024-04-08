use owo_colors::OwoColorize;
use url::Url;

use crate::log::GlobalLogger;

pub fn normalize_url(
    partial_url: &str,
    fallback: &str,
) -> String {
    // Parse the partial URL
    let url = match Url::parse(partial_url) {
        Ok(url) => {
            // If the partial URL is parsed successfully, print the parsed URL
            url
        },
        Err(_) => {
            // If there's an error parsing the URL, assume it doesn't have a scheme
            // and prepend a default scheme (e.g., "https://") before parsing again
            let default_scheme = "https://";
            let full_url = format!("{}{}", default_scheme, partial_url);
            match Url::parse(&full_url) {
                Ok(url) => {
                    // Print the parsed URL
                    url
                },
                Err(err) => {
                    GlobalLogger::warn(format!(
                        "Invalid server ({}), using fallback ({})!",
                        err.red(),
                        fallback.blue()
                    ));

                    // If there's still an error parsing the URL, print the error
                    Url::parse(fallback).unwrap()
                },
            }
        },
    };

    return url.as_str().to_owned();
}

pub trait ResultToString<T, E> {
    fn map_err_to_string(self) -> Result<T, String>;
}

impl<T, E: std::error::Error> ResultToString<T, E> for Result<T, E> {
    fn map_err_to_string(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}
