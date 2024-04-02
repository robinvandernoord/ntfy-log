// use color_eyre::eyre::Result;
use atty::Stream;
use clap::{CommandFactory, Parser};
use ntfy::{Dispatcher, Payload, Priority};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::process::Command;
use url::Url;

const DEFAULT_SERVER: &str = "https://ntfy.sh";

/// Either provide a channel and a command to run (`ntfy-log some-channel some-command --with-options`)
/// or pipe the result of a command into this tool (`some-command --with-options | ntfy-log some-channel`)
#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = (DEFAULT_SERVER).into())]
    endpoint: String,

    #[arg(short, long)]
    version: bool,

    #[arg(long)]
    self_update: bool,

    #[arg(short, long, required = false, default_value_t=String::from(""))]
    title: String,

    #[arg(required = true, num_args(1), conflicts_with_all = ["self_update", "version"])]
    topic: Option<String>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = false, num_args(0..))]
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

#[derive(Debug)]
struct InvalidArgsNoStdIn {}

/// Non-preferred way, since command, stderr and exit_code are all missing!
fn try_stdin() -> Result<CommandResult, InvalidArgsNoStdIn> {
    if atty::is(Stream::Stdin) {
        return Err(InvalidArgsNoStdIn {});
    }

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    return Ok(CommandResult {
        command: String::from("<stdin>"), // command is not known when getting data from stdin
        stdout: input,

        stderr: String::from(""), // stderr is usually not piped, unless it is combined with stdout into stdin.
        exit_code: 0, // unfortunately, you can't get the exit code of a piped command ($PIPESTATUS is bash-only)
    });
}

async fn run_cmd(args: &Vec<String>) -> Result<CommandResult, InvalidArgsNoStdIn> {
    if args.len() == 0 {
        // no subcommand arg(s), hopefully something was piped.
        return try_stdin();
    };

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
        return Err(InvalidArgsNoStdIn {});
    };

    print!("{}", result.stdout);
    eprint!("{}", result.stderr);

    return Ok(result);
}


async fn print_version() -> Result<i32, String> {
    Ok(0)
}


async fn self_update() ->  Result<i32, String> {
    Ok(0)
}


/// Main logic, but returns a Result(exit code | ntfy error) instead of exiting.
async fn main_with_exitcode() -> Result<i32, String> {
    let args = Cli::parse();

    if args.version {
        return print_version().await;
    } else if args.self_update {
        return self_update().await;
    }

    let topic = args.get_topic();

    let ntfy = setup_ntfy(&args.endpoint);

    let _result = run_cmd(&args.subcommand).await;

    if _result.is_err() {
        Cli::command()
            // .color(clap::ColorChoice::Always) // coloring does not work here for some reason (but it does for default help?)
            .print_long_help()
            .unwrap_or_default();
        return Ok(2); // not really ok but usage already printed.
    }

    let result = _result.unwrap();

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
    ntfy.send(&payload).await.map_err(|err| err.to_string())?;

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
    ntfy.send(&secondary_payload)
        .await
        .map_err(|err| err.to_string())?;

    Ok(result.exit_code)
}

/// Run main_with_exitcode and exit with the returned exit code, or print any (non-panicking) error.
#[tokio::main]
async fn main() -> ! {
    // color_eyre::install()?;

    match main_with_exitcode().await {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!(">> ntfy | {} | {}", "error".red(), error);
            std::process::exit(-1)
        }
    }
}
