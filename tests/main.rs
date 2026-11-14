mod commands;
mod utils;

use std::process::Command;

#[test]
fn requires_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-tool-check"))
        .output()
        .expect("run binary");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("<COMMAND>"));
}

#[test]
fn rejects_unknown_agent() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-tool-check"))
        .arg("cursor")
        .output()
        .expect("run binary");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unrecognized subcommand 'cursor'"));
}
