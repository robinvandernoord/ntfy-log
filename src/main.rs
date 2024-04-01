use clap::Parser;
// use color_eyre::eyre::Result;
use ntfy::{Dispatcher, NtfyError, Payload, Priority};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::process::Command;
use url::Url;

const DEFAULT_SERVER: &str = "https://ntfy.sh";

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = (DEFAULT_SERVER).into())]
    endpoint: String,

    #[arg(short, long, required = false, default_value_t=String::from(""))]
    title: String,

    #[arg(required = true, num_args(1))]
    topic: Option<String>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, required=true, num_args(1..))]
    subcommand: Vec<String>,
}

impl Cli {
    fn get_topic(&self) -> &str {
        return self.topic.as_ref().unwrap();
    }
}

fn normalize_url(partial_url: &str, fallback: &str) -> String {
    // Parse the partial URL
    let url = match Url::parse(partial_url) {
        Ok(url) => {
            // If the partial URL is parsed successfully, print the parsed URL
            url
        }
        Err(_) => {
            // If there's an error parsing the URL, assume it doesn't have a scheme
            // and prepend a default scheme (e.g., "https://") before parsing again
            let default_scheme = "https://";
            let full_url = format!("{}{}", default_scheme, partial_url);
            match Url::parse(&full_url) {
                Ok(url) => {
                    // Print the parsed URL
                    url
                }
                Err(err) => {
                    eprintln!(
                        ">> ntfy | {} | Invalid server ({}), using fallback ({})!",
                        "warn".yellow(),
                        err.red(),
                        fallback.blue()
                    );
                    // If there's still an error parsing the URL, print the error
                    Url::parse(fallback).unwrap()
                }
            }
        }
    };

    return url.as_str().to_owned();
}

fn setup_ntfy(server: &str) -> Dispatcher {
    let server_uri = normalize_url(server, DEFAULT_SERVER);

    return Dispatcher::builder(server_uri).build().unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandResult {
    command: String,

    stdout: String,
    stderr: String,
    exit_code: i32,
}

impl CommandResult {
    fn success(&self) -> bool {
        return self.exit_code == 0;
    }

    fn build_payload(&self, topic: &str) -> Payload {
        let priority = match self.success() {
            true => Priority::Default,
            false => Priority::High,
        };

        let msg = match serde_json::to_string(self) {
            Ok(msg) => msg,
            Err(error) => {
                let fallback = json!({
                    "error": error.to_string(),
                });

                fallback.to_string()
            }
        };

        return Payload::new(topic)
            .title(&self.command)
            .message(msg)
            .priority(priority);
    }
}

async fn run_cmd(args: &Vec<String>) -> CommandResult {
    let command = args.join(" ");
    eprintln!(">> ntfy | {} | {}", "info".blue(), command.blue());

    let result = if let Some((base, args)) = args.split_first() {
        let mut cmd = Command::new(base);
        cmd.args(args);

        match cmd.output().await {
            Ok(output) => CommandResult {
                command: command,
                stdout: String::from_utf8(output.stdout).unwrap_or_default(),
                stderr: String::from_utf8(output.stderr).unwrap_or_default(),
                exit_code: output.status.code().unwrap_or(-1),
            },

            Err(error) => CommandResult {
                command: command,
                stdout: String::from(""),
                stderr: error.to_string(),
                exit_code: error.raw_os_error().unwrap_or(-1),
            },
        }
    } else {
        // wtf happened?
        CommandResult {
            command: command,
            stdout: String::from(""),

            stderr: String::from("Invalid args to run_cmd"),
            exit_code: -1,
        }
    };

    print!("{}", result.stdout);
    eprint!("{}", result.stderr);

    result
}

async fn main_with_exitcode() -> Result<i32, NtfyError> {
    let args = Cli::parse();
    let topic = args.get_topic();

    let ntfy = setup_ntfy(&args.endpoint);

    let result = run_cmd(&args.subcommand).await;
    let mut payload = result.build_payload(topic);

    if args.title != "" {
        payload = payload.title(&args.title)
    }

    eprintln!(
        ">> ntfy | {} | Sending {:?} to {}",
        "info".blue(),
        payload,
        args.endpoint
    );
    ntfy.send(&payload).await?;

    // also send 'title' to the success or failure channel:
    // todo: make this an option
    let suffix = match result.success() {
        true => "success",
        false => "failure",
    };

    let secondary_topic = format!("{}--{}", topic, suffix);

    let secondary_msg = payload.title.unwrap_or_default();

    let secondary_payload = Payload::new(secondary_topic).message(&secondary_msg);

    eprintln!(
        ">> ntfy | {} | Sending {:?} to {}.",
        "info".blue(),
        secondary_payload,
        args.endpoint
    );
    ntfy.send(&secondary_payload).await?;

    Ok(result.exit_code)
}

#[tokio::main]
async fn main() -> ! {
    // color_eyre::install()?;

    match main_with_exitcode().await {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!(">> ntfy | {} | {}", "error".red(), error.to_string());
            std::process::exit(-1)
        }
    }
}
