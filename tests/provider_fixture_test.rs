#[path = "support/provider_fixture.rs"]
mod helpers;

use helpers::{blocked_reason, run_fixture};
use serde_json::Value;

#[test]
fn allows_claude_grep_fixture_with_path_field() {
    let output = run_fixture("claude", "claude/pre_tool_use/grep_path.json");
    assert!(output.status.success());
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn blocks_claude_read_fixture_for_sensitive_file() {
    let output = run_fixture("claude", "claude/pre_tool_use/read_env.json");
    let json = output_json(&output);

    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to sensitive file blocked"
    );
    assert_eq!(
        json,
        serde_json::json!({
            "decision": "block",
            "reason": "Access to sensitive file blocked",
        })
    );
}

fn output_json(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("parse stdout as JSON")
}
