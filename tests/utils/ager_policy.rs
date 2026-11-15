use std::{fs, path, process};

use serde_json::json;

use super::hook_json;

const REASON: &str = "Access to ager plaintext file blocked; use its .age file instead";
const MANIFEST: &str = "identity: ~/.config/sops/age/keys.txt\nentries:\n  - home/.config/nix/nix.conf\n  - home/.ssh/config\n  - 'private settings.txt'\n";

#[derive(Clone, Copy, Debug)]
enum Tool {
    Bash,
    Grep,
    Read,
}

pub fn blocks_plaintext(agent: &str) {
    let project = project_with_manifest(MANIFEST);
    let root = project.path();
    let absolute = root.join("home/.config/nix/nix.conf");
    let absolute_command = format!("head '{}'", absolute.display());
    for command in [
        "head home/.config/nix/nix.conf",
        "head ./home/.config/nix/nix.conf",
        "head home/other/../.config/nix/nix.conf",
        "head home/.ssh/config",
        "head home/.ssh/config.age home/.config/nix/nix.conf",
        "head home/.config/nix/nix.conf home/.ssh/config.age",
        "head 'private settings.txt'",
        "rg pattern home/.config/nix/nix.conf",
        "sed -n '1p' home/.config/nix/nix.conf",
        "head < home/.config/nix/nix.conf",
        "sh -c 'head home/.config/nix/nix.conf'",
        "echo $(head home/.config/nix/nix.conf)",
        &absolute_command,
    ] {
        assert_blocked(agent, root, Tool::Bash, command, REASON);
    }
    assert_blocked(
        agent,
        root,
        Tool::Bash,
        "head home/.ssh/config.age .env",
        "Never access .env files directly",
    );
    if agent == "codex" {
        return;
    }
    for tool in [Tool::Read, Tool::Grep] {
        for file in [
            "home/.config/nix/nix.conf",
            "./home/.config/nix/nix.conf",
            "home/.ssh/config",
            "private settings.txt",
            absolute.to_str().expect("absolute path as text"),
        ] {
            assert_blocked(agent, root, tool, file, REASON);
        }
    }
}

pub fn allows_ciphertext_and_unlisted(agent: &str) {
    let project = project_with_manifest(MANIFEST);
    let root = project.path();
    for command in [
        "head home/.config/nix/nix.conf.age",
        "head home/.ssh/config.age",
        "head 'private settings.txt.age'",
        "head public.txt",
        "head other/home/.config/nix/nix.conf",
        "echo home/.config/nix/nix.conf",
        "rg home/.config/nix/nix.conf README.md",
        "git status",
    ] {
        assert_allowed(agent, root, Tool::Bash, command);
    }
    if agent == "codex" {
        return;
    }
    for tool in [Tool::Read, Tool::Grep] {
        for file in [
            "home/.config/nix/nix.conf.age",
            "home/.ssh/config.age",
            "private settings.txt.age",
            "public.txt",
        ] {
            assert_allowed(agent, root, tool, file);
        }
    }
}

pub fn only_uses_current_project(agent: &str) {
    let project = project_with_manifest(MANIFEST);
    let child = project.path().join("child");
    fs::create_dir_all(&child).expect("create nested project");
    assert_allowed(
        agent,
        &child,
        Tool::Bash,
        "head ../home/.config/nix/nix.conf",
    );
    let unrelated = tempfile::tempdir().expect("create unrelated project");
    let unrelated_root = unrelated.path();
    assert_allowed(
        agent,
        unrelated_root,
        Tool::Bash,
        "head home/.config/nix/nix.conf",
    );
    assert_blocked(
        agent,
        unrelated_root,
        Tool::Bash,
        "head .env",
        "Never access .env files directly",
    );
}

pub fn rejects_invalid_manifests(agent: &str) {
    for manifest in [
        "entries: [",
        "entries: wrong",
        "identity: key.txt",
        "entries: []\nunknown: true",
    ] {
        let project = project_with_manifest(manifest);
        let root = project.path();
        assert_blocked(
            agent,
            root,
            Tool::Bash,
            "head public.txt",
            "failed to parse project ager.yaml",
        );
    }
    let project = tempfile::tempdir().expect("create project");
    let root = project.path();
    let manifest_path = root.join("ager.yaml");
    fs::create_dir_all(manifest_path).expect("create unreadable manifest");
    assert_blocked(
        agent,
        root,
        Tool::Bash,
        "head public.txt",
        "failed to read project ager.yaml",
    );
    let empty = project_with_manifest("entries: []");
    let empty_root = empty.path();
    assert_allowed(agent, empty_root, Tool::Bash, "head public.txt");
}

fn project_with_manifest(manifest: &str) -> tempfile::TempDir {
    let project = tempfile::tempdir().expect("create project");
    let manifest_path = project.path().join("ager.yaml");
    fs::write(manifest_path, manifest).expect("write project manifest");
    project
}

fn run(agent: &str, cwd: &path::Path, tool: Tool, value: &str) -> process::Output {
    let input = match tool {
        Tool::Bash => json!({"command": value}),
        Tool::Read if agent == "claude" => json!({"file_path": value}),
        Tool::Read | Tool::Grep => json!({"path": value}),
    };
    let payload = if agent == "pi" {
        let name = match tool {
            Tool::Bash => "bash",
            Tool::Read => "read",
            Tool::Grep => "grep",
        };
        json!({"toolName": name, "input": input})
    } else {
        let name = match tool {
            Tool::Bash => "Bash",
            Tool::Read => "Read",
            Tool::Grep => "Grep",
        };
        json!({"cwd": cwd, "tool_name": name, "tool_input": input})
    };
    let payload = payload.to_string();
    hook_json::run_pre_tool_use_json_in(agent, &payload, cwd)
}

fn assert_blocked(agent: &str, cwd: &path::Path, tool: Tool, value: &str, reason: &str) {
    let output = run(agent, cwd, tool, value);
    assert!(output.status.success(), "hook should exit successfully");
    assert_eq!(
        hook_json::blocked_reason(&output).as_deref(),
        Some(reason),
        "{agent} {tool:?}: {value}"
    );
}

fn assert_allowed(agent: &str, cwd: &path::Path, tool: Tool, value: &str) {
    let output = run(agent, cwd, tool, value);
    assert!(output.status.success(), "hook should exit successfully");
    assert!(
        output.stdout.is_empty(),
        "{agent} {tool:?}: {value}: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
