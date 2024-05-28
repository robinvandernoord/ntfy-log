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
    pub const fn success(&self) -> bool {
        self.exit_code == 0
    }

    pub fn build_payload(
        &self,
        topic: &str,
    ) -> Payload {
        let priority = if self.success() {
            Priority::Default
        } else {
            Priority::High
        };

        let msg = serde_json::to_string(self).unwrap_or_else(|error| {
            let fallback = json!({
                "error": error.to_string(),
            });

            fallback.to_string()
        });

        Payload::new(topic)
            .title(&self.command)
            .message(msg)
            .priority(priority)
    }
}

#[derive(Debug)]
pub struct InvalidArgsNoStdIn {}

/// Non-preferred way, since `command`, `stderr` and `exit_code` are all missing!
pub fn try_stdin() -> Result<CommandResult, InvalidArgsNoStdIn> {
    if atty::is(Stream::Stdin) {
        return Err(InvalidArgsNoStdIn {});
    }

    GlobalLogger::important("warn".yellow().to_string(),
                            "Since complex bash commands (including pipes) is now supported by ntfy-push, using stdin is highly discouraged.");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    Ok(CommandResult {
        command: String::from("<stdin>"), // command is not known when getting data from stdin
        stdout: input,

        stderr: String::new(), // stderr is usually not piped, unless it is combined with stdout into stdin.
        exit_code: 0, // unfortunately, you can't get the exit code of a piped command ($PIPESTATUS is bash-only)
    })
}

pub async fn run_cmd(args: &[String]) -> Result<CommandResult, InvalidArgsNoStdIn> {
    let logger = GlobalLogger::singleton();

    if args.is_empty() {
        // no subcommand arg(s), hopefully something was piped.
        return try_stdin();
    };

    let command = args.join(" ");
    logger.info(command.blue().to_string());

    if command.is_empty() {
        return Err(InvalidArgsNoStdIn {});
    }

    let mut cmd = Command::new("bash");
    cmd.arg("-c");
    cmd.arg(&command);
    // -> bash -c "<full command>"

    let result = match cmd.output().await {
        Ok(output) => CommandResult {
            command,
            stdout: String::from_utf8(output.stdout).unwrap_or_default(),
            stderr: String::from_utf8(output.stderr).unwrap_or_default(),
            exit_code: output.status.code().unwrap_or(-1),
        },

        Err(error) => CommandResult {
            command,
            stdout: String::new(),
            stderr: error.to_string(),
            exit_code: error.raw_os_error().unwrap_or(-1),
        },
    };

    logger.stdout(&result.stdout);
    logger.stderr(&result.stderr);

    Ok(result)
}
