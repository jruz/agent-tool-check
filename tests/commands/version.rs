use std::process::Command;

#[test]
fn version_entry_points_match() {
    let outputs = ["version", "--version"].map(|argument| {
        Command::new(env!("CARGO_BIN_EXE_agent-tool-check"))
            .arg(argument)
            .output()
            .expect("run binary")
    });
    for output in &outputs {
        assert!(output.status.success());
        assert_eq!(output.stderr, b"");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.starts_with("agent-tool-check "));
        assert!(text.contains(concat!("\nversion: ", env!("CARGO_PKG_VERSION"))));
        assert!(text.contains("\nrevision: "));
        assert!(text.contains("\nbuild-time: "));
    }
    let [command, flag] = outputs;
    assert_eq!(command.stdout, flag.stdout);
}
