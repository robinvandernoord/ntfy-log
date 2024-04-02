pub use ntfy::{Dispatcher, Payload};

use crate::constants::DEFAULT_NTFY_SERVER;
use crate::helpers::normalize_url;

pub fn setup_ntfy(server: &str) -> Dispatcher {
    let server_uri = normalize_url(server, DEFAULT_NTFY_SERVER);

    return Dispatcher::builder(server_uri).build().unwrap();
}
