mod cli;
mod command;
mod constants;
mod helpers;
mod http;
mod ntfy;
mod self_update;

use clap::{CommandFactory, Parser};
use owo_colors::OwoColorize;

use self::cli::Cli;
use self::command::run_cmd;
use self::ntfy::{setup_ntfy, Payload};
use self::self_update::{current_version, pkg_name, self_update};

async fn print_version() -> Result<i32, String> {
    println!("{} {}", pkg_name(), current_version());
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
