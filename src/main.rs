mod pre_tool_use;
mod shared;
mod version;

use std::collections::HashMap;

use clap::Parser;

use crate::shared::agent::Agent;

#[derive(Debug, Parser)]
#[command(version = version::clap_version())]
struct Cli {
    agent: Agent,
}

fn main() {
    let cli = Cli::parse();

    match pre_tool_use::run(cli.agent) {
        Ok(Some(output)) => println!("{output}"),
        Ok(None) => {}
        Err(err) => {
            let reason = err.to_string();
            let attributes = pre_tool_use::log_attributes(&err);
            let log_result = log_pre_tool_use_error(attributes, &reason);
            let log_error = log_result.err();
            if let Some(log_err) = log_error {
                eprintln!("logging failed: {log_err}");
            }
            eprintln!("{reason}");
        }
    }
}

fn log_pre_tool_use_error(
    mut attributes: HashMap<String, String>,
    reason: &str,
) -> Result<(), shared::logger::Error> {
    let key = String::from("reason");
    let value = reason.to_owned();
    attributes.insert(key, value);
    shared::logger::error("PreToolUse", attributes)
}
