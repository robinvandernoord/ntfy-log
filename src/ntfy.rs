pub use ntfy::{Dispatcher, Payload};

use crate::constants::DEFAULT_NTFY_SERVER;
use crate::helpers::normalize_url;
use crate::log::Logger;

pub fn setup_ntfy(server: &str, logger: &Logger) -> Dispatcher {
    let server_uri = normalize_url(server, DEFAULT_NTFY_SERVER, &logger);

    return Dispatcher::builder(server_uri).build().unwrap();
}
