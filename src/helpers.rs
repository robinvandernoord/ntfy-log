use crate::constants::DEFAULT_SCHEMA;
use crate::log::GlobalLogger;
use owo_colors::OwoColorize;
use url::Url;

pub fn normalize_url(
    partial_url: &str,
    fallback: &str,
) -> String {
    // Parse the partial URL

    let url = Url::parse(partial_url).unwrap_or_else(|_| {
        // If there's an error parsing the URL, assume it doesn't have a scheme
        // and prepend a default scheme (e.g., "https://") before parsing again
        let full_url = format!("{DEFAULT_SCHEMA}{partial_url}");

        Url::parse(&full_url).unwrap_or_else(|err| {
            GlobalLogger::warn(format!(
                "Invalid server ({}), using fallback ({})!",
                err.red(),
                fallback.blue()
            ));

            // If there's still an error parsing the URL, print the error
            Url::parse(fallback).unwrap()
        })
    });

    url.as_str().to_owned()
}

pub trait ResultToString<T, E> {
    fn map_err_to_string(self) -> Result<T, String>;
}

impl<T, E: std::error::Error> ResultToString<T, E> for Result<T, E> {
    fn map_err_to_string(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}
