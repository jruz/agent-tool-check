use std::io::Write as _;
use std::process::Command;

#[test]
fn requires_agent() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-tool-check"))
        .output()
        .expect("run binary");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("<AGENT>"));
}

#[test]
fn rejects_unknown_agent() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-tool-check"))
        .arg("cursor")
        .output()
        .expect("run binary");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid value 'cursor'"));
}

#[test]
fn logs_pre_tool_use_agent() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let state_home = temp.path().join("state");
    let mut stdin_file = tempfile::NamedTempFile::new().expect("create stdin temp file");
    stdin_file.write_all(b"{").expect("write stdin");
    let stdin_path = stdin_file.into_temp_path();
    let stdin = std::fs::File::open(stdin_path).expect("open stdin temp file");

    let output = Command::new(env!("CARGO_BIN_EXE_agent-tool-check"))
        .arg("codex")
        .env("XDG_STATE_HOME", &state_home)
        .stdin(stdin)
        .output()
        .expect("run binary");

    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("failed to parse Codex PreToolUse JSON")
    );

    let log_path = state_home.join("agent-tool-check").join("hooks.log");
    let log = std::fs::read_to_string(log_path).expect("read log");
    assert!(log.contains(r#""Body":"PreToolUse""#));
    assert!(log.contains(r#""agent":"codex""#));
    assert!(log.contains(r#""stdin_bytes":"1""#));
    assert!(log.contains(r#""json_valid":"false""#));
    assert!(log.contains(r#""json_error":"EOF while parsing an object at line 1 column 1""#));
    assert!(log.contains(r#""reason":"failed to parse Codex PreToolUse JSON"#));
}

#[test]
fn logs_pre_tool_use_payload_shape() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let state_home = temp.path().join("state");
    let stdin_json = r#"{"cwd":"/tmp","hook_event_name":"PreToolUse","session_id":"test","tool_input":{"unexpected":"value"},"tool_name":"Read","tool_use_id":"test-id","transcript_path":"/tmp/test"}"#;
    let mut stdin_file = tempfile::NamedTempFile::new().expect("create stdin temp file");
    let stdin_bytes = stdin_json.as_bytes();
    stdin_file.write_all(stdin_bytes).expect("write stdin");
    let stdin_path = stdin_file.into_temp_path();
    let stdin = std::fs::File::open(stdin_path).expect("open stdin temp file");

    let output = Command::new(env!("CARGO_BIN_EXE_agent-tool-check"))
        .arg("claude")
        .env("XDG_STATE_HOME", &state_home)
        .stdin(stdin)
        .output()
        .expect("run binary");

    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("failed to parse Claude PreToolUse JSON")
    );

    let log_path = state_home.join("agent-tool-check").join("hooks.log");
    let log = std::fs::read_to_string(log_path).expect("read log");
    let top_level_keys = r#""top_level_keys":"cwd,hook_event_name,session_id,tool_input,tool_name,tool_use_id,transcript_path""#;
    assert!(log.contains(r#""agent":"claude""#));
    assert!(log.contains(r#""json_valid":"true""#));
    assert!(log.contains(r#""hook_event_name":"PreToolUse""#));
    assert!(log.contains(r#""tool_name":"Read""#));
    assert!(log.contains(r#""tool_input_kind":"object""#));
    assert!(log.contains(r#""tool_input_keys":"unexpected""#));
    assert!(log.contains(top_level_keys));
}
