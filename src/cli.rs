// use color_eyre::eyre::Result;

use clap::Parser;

use crate::constants::DEFAULT_NTFY_SERVER;

/// Either provide a channel and a command to run (`ntfy-log some-channel some-command --with-options`)
/// or pipe the result of a command into this tool (`some-command --with-options | ntfy-log some-channel`)
#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short, long, default_value_t = (DEFAULT_NTFY_SERVER).into())]
    pub endpoint: String,

    #[arg(short = 'V', long)]
    pub version: bool,

    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    #[arg(long)]
    pub self_update: bool,

    #[arg(short, long, required = false, default_value_t=String::from(""))]
    pub title: String,

    #[arg(required = true, num_args(1), conflicts_with_all = ["self_update", "version"])]
    topic: Option<String>, // private, use get_topic instead!

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = false, num_args(0..))]
    pub subcommand: Vec<String>,
}

impl Cli {
    pub fn get_topic(&self) -> &str {
        self.topic
            .as_ref()
            .expect("topic is marked as `required = true` so we can assume it's there.")
    }
}
