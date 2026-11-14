mod commands;
mod core;
mod libs;
mod utils;

use std::collections::HashMap;
use std::process::ExitCode;

use clap::Parser as _;

use crate::commands::version;
use crate::utils::{failure, logger, output};

#[derive(Debug, clap::Parser)]
#[command(version = version::text())]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, Debug, clap::Subcommand)]
enum Command {
    Claude,
    Codex,
    Pi,
    #[command(about = "Show version and build metadata")]
    Version,
}

fn run(command: Command) -> Result<output::Output, failure::Error> {
    match command {
        Command::Claude => commands::claude::run(),
        Command::Codex => commands::codex::run(),
        Command::Pi => commands::pi::run(),
        Command::Version => {
            let output = version::run();
            Ok(output)
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let output = match run(cli.command) {
        Ok(output) => output,
        Err(err) => {
            let reason = err.to_string();
            let attributes = err.attributes;
            if let Err(log_err) = log_pre_tool_use_error(attributes, &reason) {
                eprintln!("logging failed: {log_err}");
            }
            eprintln!("{reason}");
            return ExitCode::SUCCESS;
        }
    };
    print!("{}", output.text);
    if output.success {
        return ExitCode::SUCCESS;
    }
    ExitCode::FAILURE
}

fn log_pre_tool_use_error(
    mut attributes: HashMap<String, String>,
    reason: &str,
) -> Result<(), logger::Error> {
    let key = String::from("reason");
    let value = reason.to_owned();
    attributes.insert(key, value);
    logger::error("PreToolUse", attributes)
}
