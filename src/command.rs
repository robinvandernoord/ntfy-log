use atty::Stream;
use ntfy::{Payload, Priority};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::process::Command;

use crate::log::GlobalLogger;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: String,

    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CommandResult {
    pub fn success(&self) -> bool {
        return self.exit_code == 0;
    }

    pub fn build_payload(&self, topic: &str) -> Payload {
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
pub struct InvalidArgsNoStdIn {}

/// Non-preferred way, since command, stderr and exit_code are all missing!
pub fn try_stdin() -> Result<CommandResult, InvalidArgsNoStdIn> {
    if atty::is(Stream::Stdin) {
        return Err(InvalidArgsNoStdIn {});
    }

    GlobalLogger::important("warn".yellow().to_string(), 
    "Since complex bash commands (including pipes) is now supported by ntfy-push, using stdin is highly discouraged.");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    return Ok(CommandResult {
        command: String::from("<stdin>"), // command is not known when getting data from stdin
        stdout: input,

        stderr: String::from(""), // stderr is usually not piped, unless it is combined with stdout into stdin.
        exit_code: 0, // unfortunately, you can't get the exit code of a piped command ($PIPESTATUS is bash-only)
    });
}

pub async fn run_cmd(args: &Vec<String>) -> Result<CommandResult, InvalidArgsNoStdIn> {
    let logger = GlobalLogger::singleton();

    if args.len() == 0 {
        // no subcommand arg(s), hopefully something was piped.
        return try_stdin();
    };

    let command = args.join(" ");
    logger.info(command.blue().to_string());

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

    logger.stdout(&result.stdout);
    logger.stderr(&result.stderr);

    return Ok(result);
}
