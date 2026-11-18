use std::fs;
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path;
use std::process;

use serde_json::{Value, json};

#[derive(serde::Deserialize)]
struct Entry {
    agent: String,
    #[serde(rename = "callId")]
    call_id: Option<String>,
    command: Option<String>,
    executable: Option<String>,
    project: String,
    reason: String,
    session: Option<String>,
    timestamp: String,
}

fn run(agent: &str, input: &Value, state: &path::Path, cwd: &path::Path) -> process::Output {
    let mut file = tempfile::NamedTempFile::new().expect("temp stdin");
    let text = input.to_string();
    let bytes = text.as_bytes();
    file.write_all(bytes).expect("write payload");
    let path = file.into_temp_path();
    let stdin_file = fs::File::open(path).expect("open stdin");
    let stdin = process::Stdio::from(stdin_file);
    let binary = env!("CARGO_BIN_EXE_agent-tool-check");
    process::Command::new(binary)
        .arg(agent)
        .current_dir(cwd)
        .env("XDG_STATE_HOME", state)
        .stdin(stdin)
        .output()
        .expect("run hook")
}

fn payload(agent: &str, command: &str) -> Value {
    if agent == "pi" {
        return json!({"toolName": "bash", "toolCallId": "call-1", "input": {"command": command}});
    }
    json!({"tool_name": "Bash", "session_id": "session-1", "tool_use_id": "call-1", "cwd": "/project", "tool_input": {"command": command}})
}

fn entries(state: &path::Path) -> Vec<Entry> {
    let path = state.join("agent-tool-check/denial.log");
    fs::read_to_string(path)
        .expect("read denial log")
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid JSONL"))
        .collect()
}

#[test]
fn captures_self_contained_denials_for_every_agent() {
    for agent in ["claude", "codex", "pi"] {
        let temp = tempfile::tempdir().expect("temp dir");
        let state = temp.path().join("state");
        let input = payload(agent, "cat .env");
        let cwd = temp.path();
        let output = run(agent, &input, &state, cwd);
        assert!(output.status.success());
        assert_eq!(output.stderr, b"");
        let response: Value = serde_json::from_slice(&output.stdout).expect("block response");
        let records = entries(&state);
        let entry = records.first().expect("one denial");
        assert_eq!(records.len(), 1);
        assert_eq!(entry.agent, agent);
        assert_eq!(entry.command.as_deref(), Some("cat .env"));
        assert_eq!(entry.executable.as_deref(), Some("cat"));
        assert_eq!(entry.call_id.as_deref(), Some("call-1"));
        let reason = response
            .get("reason")
            .or_else(|| {
                response
                    .get("hookSpecificOutput")?
                    .get("permissionDecisionReason")
            })
            .and_then(Value::as_str)
            .expect("response reason");
        assert_eq!(entry.reason, reason);
        chrono::DateTime::parse_from_rfc3339(&entry.timestamp).expect("RFC3339 timestamp");
        if agent == "pi" {
            let project = temp.path().to_str().expect("UTF8 path");
            assert_eq!(entry.project, project);
            assert!(entry.session.is_none());
        } else {
            assert_eq!(entry.project, "/project");
            assert_eq!(entry.session.as_deref(), Some("session-1"));
        }
        assert!(!state.join("agent-tool-check/errors.log").exists());
        assert!(!state.join("agent-tool-check/hooks.log").exists());
    }
}

#[test]
fn allows_do_not_create_denial_logs() {
    for agent in ["claude", "codex", "pi"] {
        let temp = tempfile::tempdir().expect("temp dir");
        let state = temp.path().join("state");
        let input = payload(agent, "git status");
        let cwd = temp.path();
        let output = run(agent, &input, &state, cwd);
        assert_eq!(output.stdout, b"");
        assert_eq!(output.stderr, b"");
        assert!(!state.exists());
    }
}

#[test]
fn denials_append_and_redact_before_persistence() {
    let temp = tempfile::tempdir().expect("temp dir");
    let state = temp.path().join("state");
    let command = "TOKEN=first PASSWORD='second value' cat .env; curl --api-key third --header 'Authorization: fourth' --user fifth https://user:sixth@example.com";
    for agent in ["claude", "codex", "pi"] {
        let input = payload(agent, command);
        let cwd = temp.path();
        let output = run(agent, &input, &state, cwd);
        assert_eq!(output.stderr, b"");
    }
    let records = entries(&state);
    assert_eq!(records.len(), 3);
    for entry in records {
        let text = entry.command.expect("command");
        for secret in ["first", "second", "third", "fourth", "fifth", "sixth"] {
            assert!(!text.contains(secret), "unredacted credential");
        }
        assert!(text.contains("cat .env"));
        assert!(text.contains("[REDACTED]"));
    }
    let path = state.join("agent-tool-check/denial.log");
    let mode = fs::metadata(path).expect("metadata").permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn captures_file_access_targets() {
    let temp = tempfile::tempdir().expect("temp dir");
    let state = temp.path().join("state");
    for input in [
        json!({"toolName": "read", "input": {"path": "/project/.env"}}),
        json!({"toolName": "grep", "input": {"path": "/project/.env"}}),
    ] {
        let cwd = temp.path();
        let output = run("pi", &input, &state, cwd);
        assert_eq!(output.stderr, b"");
    }
    let records = entries(&state);
    assert_eq!(records.len(), 2);
    for entry in records {
        let invocation = entry.command.expect("invocation");
        assert!(invocation.contains("/project/.env"));
        assert!(entry.executable.is_none());
    }
}

#[test]
fn logging_failures_do_not_remove_denials() {
    let temp = tempfile::tempdir().expect("temp dir");
    let state = temp.path().join("not-a-directory");
    fs::write(&state, "occupied").expect("block state path");
    for agent in ["claude", "codex", "pi"] {
        let input = payload(agent, "cat .env");
        let cwd = temp.path();
        let output = run(agent, &input, &state, cwd);
        assert_ne!(output.stdout, b"");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("denial logging failed"));
    }
}

#[test]
fn invalid_audit_metadata_does_not_remove_denials() {
    for agent in ["claude", "codex", "pi"] {
        let temp = tempfile::tempdir().expect("temp dir");
        let state = temp.path().join("state");
        let mut input = payload(agent, "cat .env");
        let metadata = input.as_object_mut().expect("payload object");
        let key = if agent == "pi" {
            "sessionId"
        } else {
            "session_id"
        };
        let key = key.to_owned();
        let invalid_session = Value::Bool(false);
        metadata.insert(key, invalid_session);
        let cwd = temp.path();
        let output = run(agent, &input, &state, cwd);
        assert_ne!(output.stdout, b"");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("denial logging failed"));
    }
}

#[test]
fn malformed_hooks_only_write_errors_log() {
    for agent in ["claude", "codex", "pi"] {
        let temp = tempfile::tempdir().expect("temp dir");
        let state = temp.path().join("state");
        let input = json!({});
        let cwd = temp.path();
        let output = run(agent, &input, &state, cwd);
        assert_eq!(output.stdout, b"");
        assert_ne!(output.stderr, b"");
        let path = state.join("agent-tool-check/errors.log");
        let text = fs::read_to_string(path).expect("error log");
        let line = text.trim();
        let entry: Value = serde_json::from_str(line).expect("error JSON");
        assert_eq!(
            entry.get("SeverityText").and_then(Value::as_str),
            Some("ERROR")
        );
        assert!(!state.join("agent-tool-check/denial.log").exists());
        assert!(!state.join("agent-tool-check/hooks.log").exists());
    }
}
