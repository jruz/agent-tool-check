use std::collections::HashMap;
use std::io::Write as _;
use std::process::{Command, Output};

use serde_json::Value;

use crate::utils::claude_hook as helpers;
use crate::utils::provider_fixture::run_fixture;
use helpers::{blocked_reason, payload, run_pre_tool_use};

const SHORT_FLAG_PREFIXES: [&str; 16] = [
    "", "a", "s", "n", "as", "an", "sa", "sn", "na", "ns", "asn", "ans", "san", "sna", "nas", "nsa",
];

fn bash(command: &str) -> helpers::PreToolUsePayload {
    let mut input = HashMap::new();
    let key = "command".to_string();
    let command_value = command.to_string();
    let value = Value::String(command_value);
    input.insert(key, value);
    payload("Bash", input)
}

fn run_bash(command: &str) -> Output {
    let input = bash(command);
    run_pre_tool_use(&input)
}

#[test]
fn bash_rules_blocks_cargo_install_path() {
    let output = run_bash("cargo install --path .");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Use `just install` instead"
    );
}

#[test]
fn bash_rules_blocks_cargo_install_registry() {
    let output = run_bash("cargo install ripgrep");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_git_c() {
    let output = run_bash("git -C /some/path status");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_git_status() {
    let output = run_bash("git status");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_npm_run() {
    let output = run_bash("npm run build");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Use `just <task>` instead of `npm run`"
    );
}

#[test]
fn bash_rules_blocks_npm_install() {
    let output = run_bash("npm install lodash");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_npx_playwright_install() {
    let output = run_bash("npx playwright install");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Use `just playwright-install` instead"
    );
}

#[test]
fn bash_rules_blocks_npx_playwright_test() {
    let output = run_bash("npx playwright test");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_npx_run() {
    let output = run_bash("npx run build");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Use `just <task>` instead of `npx run`"
    );
}

#[test]
fn bash_rules_blocks_skip_hooks() {
    let output = run_bash("SKIP_HOOKS=1 just check");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Don't skip hooks"
    );
}

#[test]
fn bash_rules_blocks_docker_exec_env() {
    let output = run_bash("docker exec mycontainer env");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Never read container environment variables"
    );
}

#[test]
fn bash_rules_allows_docker_exec_ls() {
    let output = run_bash("docker exec mycontainer ls");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_age_decrypt() {
    let output = run_bash("age --decrypt secret.age");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to sensitive command blocked"
    );
}

#[test]
fn bash_rules_blocks_sops_decrypt() {
    let output = run_bash("sops --decrypt secrets.yaml");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Direct SOPS decryption is blocked. Read the encrypted YAML directly to inspect keys and unencrypted values. Use `just sops-decrypt-dev` to generate `.env` and `just dotenv-show` for masked inspection, where these recipes exist."
    );
}

#[test]
fn bash_rules_blocks_sops_short_decrypt() {
    let output = run_bash("sops -d secrets.yaml");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Direct SOPS decryption is blocked. Read the encrypted YAML directly to inspect keys and unencrypted values. Use `just sops-decrypt-dev` to generate `.env` and `just dotenv-show` for masked inspection, where these recipes exist."
    );
}

#[test]
fn bash_rules_blocks_env_file() {
    let output = run_bash("cat .env");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Never access .env files directly"
    );
}

#[test]
fn bash_rules_blocks_env_in_path() {
    let output = run_bash("source /app/.env");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Never access .env files directly"
    );
}

#[test]
fn bash_rules_allows_cat_envrc() {
    let output = run_bash("cat .envrc");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_write_envrc() {
    let output = run_bash("echo 'use flake' > .envrc");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_read_env_example() {
    let output = run_bash("cat .env.example");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_git_add_env() {
    let output = run_bash("git add .env");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Never access .env files directly"
    );
}

#[test]
fn bash_rules_allows_git_commit_env_message() {
    let output = run_bash(r#"git commit -m "[feat] Env: update .env""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_gleam_run() {
    let output = run_bash("gleam run");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Use just run instead"
    );
}

#[test]
fn bash_rules_blocks_bare_gleam_build() {
    let output = run_bash("gleam build");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_sandbox_gleam_build() {
    let output = run_bash("sandbox gleam build");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_gleam_run_with_redirect() {
    let output = run_bash("gleam run 2>&1");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Use just run instead"
    );
}

#[test]
fn bash_rules_allows_gleam_run_mid_command() {
    let output = run_bash("echo gleam run");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_security_command() {
    let output = run_bash("security find-generic-password");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to macOS security command blocked"
    );
}

#[test]
fn bash_rules_blocks_update_snapshots() {
    let output = run_bash("npx playwright test --update-snapshots");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_update_snapshots_underscore() {
    let output = run_bash("pytest --update_snapshots");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_git_filter_branch_all() {
    let output = run_bash(
        "git filter-branch --force --index-filter 'git rm --cached --ignore-unmatch PLAN.md' --prune-empty -- --all",
    );
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_git_filter_branch_specific() {
    let output = run_bash(
        "git filter-branch --force --index-filter 'git rm --cached --ignore-unmatch PLAN.md' --prune-empty -- main",
    );
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_ls() {
    let output = run_bash("ls -la");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_just_build() {
    let output = run_bash("just build");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_bare_cargo() {
    let output = run_bash("cargo build");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Must be prefixed by `sandbox` (e.g. `sandbox cargo build`). Use `just` recipes when available."
    );
}

#[test]
fn bash_rules_blocks_bare_bun() {
    let output = run_bash("bun install");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_bare_node() {
    let output = run_bash("node script.js");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_node_in_commit_message() {
    let output = run_bash(r#"git commit -m "[bugs] Node: improve lookup""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_node_in_single_quoted_argument() {
    let output = run_bash("echo 'node in text'");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_node_as_an_argument() {
    let output = run_bash("rg node src");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_node_in_comment() {
    let output = run_bash("git status # node is not executed");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_node_in_command_substitution() {
    let output = run_bash(r#"git commit -m "$(node message.js)""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_node_in_backquoted_substitution() {
    let output = run_bash("git commit -m \"`node message.js`\"");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_single_quoted_command_substitution_text() {
    let output = run_bash("echo '$(node message.js)'");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_node_in_process_substitution() {
    let output = run_bash("diff <(node first.js) expected.txt");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_node_after_pipeline() {
    let output = run_bash("echo input | node script.js");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_node_in_subshell() {
    let output = run_bash("(node script.js)");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_node_in_function_body() {
    let output = run_bash("run() { node script.js; }; run");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_dynamic_executable() {
    let output = run_bash("runner=node; \"$runner\" script.js");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_globbed_executable() {
    let output = run_bash("./no* script.js");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_brace_expanded_executable() {
    let output = run_bash("no{de,thing} script.js");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_node_through_eval() {
    let output = run_bash("eval 'node script.js'");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_safe_eval() {
    let output = run_bash("eval 'git status'");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_dynamic_eval() {
    let output = run_bash("eval \"$command\"");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_node_through_env() {
    let output = run_bash("env MODE=test node script.js");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_node_through_env_argv0() {
    let output = run_bash("env -a fake node script.js");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_git_through_env_argv0() {
    let output = run_bash("env -a fake git status");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_node_through_command() {
    let output = run_bash("command node script.js");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_node_through_exec() {
    let output = run_bash("exec node script.js");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_node_through_find_exec() {
    let output = run_bash("find . -exec node script.js {} \\;");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_sandboxed_node_through_find_exec() {
    let output = run_bash("find . -exec sandbox node script.js {} \\;");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_xargs_launcher() {
    let output = run_bash("printf input | xargs rg");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_absolute_node_path() {
    let output = run_bash("/usr/bin/node script.js");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_malformed_shell_command() {
    let output = run_bash("echo ready &&");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_bare_python() {
    let output = run_bash("python script.py");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_bare_python3() {
    let output = run_bash("python3 script.py");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_bare_pnpm() {
    let output = run_bash("pnpm install");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_bare_deno() {
    let output = run_bash("deno run script.ts");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_sandbox_cargo() {
    let output = run_bash("sandbox cargo build");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_sandbox_bun() {
    let output = run_bash("sandbox bun install");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_sandbox_node() {
    let output = run_bash("sandbox node script.js");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_relative_sandbox_executable() {
    let output = run_bash("./sandbox node script.js");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_substitution_before_sandbox_node() {
    let output = run_bash(r#"sandbox node "$(node generate.js)""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_bare_zsh() {
    let output = run_bash("zsh -c 'echo ready'");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_node_nested_in_sh() {
    let output = run_bash("sh -c 'node script.js'");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_sandbox_zsh() {
    let output = run_bash("sandbox zsh -c 'node script.js'");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_bare_nu() {
    let output = run_bash("nu -c '^git status'");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_nu_recipe_syntax_check() {
    let output = run_bash(
        "nu --commands '^just --show apple-sync-logs-clear | lines | skip 2 | str join (char newline) | nu-check --debug'",
    );
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_sandbox_nu() {
    let output = run_bash("sandbox nu -c '^node script.js'");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_node_through_env_split_string() {
    let output = run_bash("env -S 'node script.js'");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_node_through_attached_env_split_string() {
    let output = run_bash("env --split-string='node script.js'");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_node_through_find_env() {
    let output = run_bash("find . -exec env node script.js {} \\;");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_dynamic_find_path() {
    let output = run_bash("find \"$directory\" -exec git status {} \\;");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_dynamic_env_assignment() {
    let output = run_bash("env MODE=\"$mode\" git status");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_quoted_heredoc_node_text() {
    let output = run_bash("cat <<'EOF'\nnode script.js\nEOF");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_node_substitution_in_expandable_heredoc() {
    let output = run_bash("cat <<EOF\n$(node script.js)\nEOF");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_npm_nested_in_nix_shell() {
    let output = run_bash("nix-shell -p nodejs --run 'npm ci'");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_dynamic_nix_shell_package() {
    let output = run_bash("nix-shell -p \"$package\" --run 'git status'");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_cargo_nested_in_bash() {
    let output = run_bash("bash -c 'cargo build'");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_npm_after_command_separator() {
    let output = run_bash("echo ready && npm ci");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_npm_after_env() {
    let output = run_bash("env npm ci");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_blocks_absolute_npm_path() {
    let output = run_bash("/usr/bin/npm ci");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn bash_rules_allows_sandboxed_npm_nested_in_nix_shell() {
    let output = run_bash("nix-shell --run 'sandbox npm ci'");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_sandboxed_absolute_cargo_path() {
    let output = run_bash("sandbox /usr/bin/cargo build");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_allows_executable_inside_cargo_directory() {
    let output = run_bash("/home/user/.cargo/bin/agent-tool-check --version");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_cat_etc_shadow() {
    let output = run_bash("cat /etc/shadow");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to system secret file blocked"
    );
}

#[test]
fn bash_rules_blocks_cat_etc_passwd() {
    let output = run_bash("cat /etc/passwd");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to system secret file blocked"
    );
}

#[test]
fn bash_rules_blocks_grep_etc_shadow() {
    let output = run_bash("grep root /etc/shadow");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to system secret file blocked"
    );
}

#[test]
fn bash_rules_blocks_ssh_cat_etc_shadow() {
    let output = run_bash("ssh user@host cat /etc/shadow");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to system secret file blocked"
    );
}

#[test]
fn bash_rules_blocks_git_commit_with_co_authored_by() {
    let output = run_bash(
        r#"git commit -m "[feat] Auth: add login" --trailer "Co-Authored-By: Claude <noreply@anthropic.com>""#,
    );
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Never include Co-Authored-By in commit messages"
    );
}

#[test]
fn bash_rules_allows_git_commit_without_co_authored_by() {
    let output = run_bash(r#"git commit -m "[feat] Auth: add login""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn bash_rules_blocks_shell_history_commands() {
    for command in [
        "history",
        "fc -l",
        "atuin search foo",
        "atuin history list",
        "atuin stats",
        "atuin key",
        "history | tail",
    ] {
        let output = run_bash(command);
        assert_eq!(
            blocked_reason(&output).expect("should block"),
            "Access to shell history blocked",
            "{command}"
        );
    }
}

#[test]
fn bash_rules_allows_atuin_maintenance_commands() {
    for command in [
        "atuin --version",
        "atuin sync",
        "atuin doctor",
        "cat ~/.config/atuin/config.toml",
    ] {
        let output = run_bash(command);
        assert!(blocked_reason(&output).is_none(), "{command}");
    }
}

#[test]
fn bash_rules_blocks_shell_history_file_operands() {
    for command in [
        "cat ~/.zsh_history",
        "tail /home/user/.bash_history",
        "rg token ~/.config/nushell/history.txt",
        "sqlite3 ~/.local/share/atuin/history.db",
    ] {
        let output = run_bash(command);
        assert_eq!(
            blocked_reason(&output).expect("should block"),
            "Access to sensitive file blocked",
            "{command}"
        );
    }
}

fn file_payload(tool_name: &str, file_path: &str) -> helpers::PreToolUsePayload {
    let mut input = HashMap::new();
    let key = "file_path".to_string();
    let file_path_value = file_path.to_string();
    let value = Value::String(file_path_value);
    input.insert(key, value);
    payload(tool_name, input)
}

fn run_file(tool_name: &str, file_path: &str) -> Output {
    let input = file_payload(tool_name, file_path);
    run_pre_tool_use(&input)
}

#[test]
fn block_secrets_blocks_read_env_file() {
    let output = run_file("Read", ".env");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to sensitive file blocked"
    );
}

#[test]
fn block_secrets_blocks_read_ssh_key() {
    let output = run_file("Read", "/home/user/.ssh/id_rsa");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to sensitive file blocked"
    );
}

#[test]
fn block_secrets_blocks_read_docker_credentials() {
    for path in [
        ".docker/config.json",
        "~/.docker/config.json",
        "/home/user/.docker/config.json",
        "/home/user/.docker/config.json.bak",
    ] {
        let output = run_file("Read", path);
        assert_eq!(
            blocked_reason(&output).expect("should block"),
            "Access to sensitive file blocked",
            "{path}"
        );
    }
}

#[test]
fn block_secrets_blocks_grep_docker_credentials_and_directory() {
    for path in ["/home/user/.docker/config.json", "/home/user/.docker"] {
        let mut input = HashMap::new();
        let path_value = path.to_string();
        let value = Value::String(path_value);
        let key = "path".to_string();
        input.insert(key, value);
        let grep_payload = payload("Grep", input);
        let output = run_pre_tool_use(&grep_payload);
        assert_eq!(
            blocked_reason(&output).expect("should block"),
            "Access to sensitive file blocked",
            "{path}"
        );
    }
}

#[test]
fn block_secrets_allows_read_unrelated_docker_files() {
    for path in [".dockerignore", "docs/docker/config.json", "config.json"] {
        let output = run_file("Read", path);
        assert!(blocked_reason(&output).is_none(), "{path}");
    }
}

#[test]
fn block_secrets_blocks_read_shell_history() {
    for path in [
        "/home/user/.bash_history",
        "~/.zsh_history",
        "/home/user/.config/nushell/history.txt",
        "/home/user/.config/nushell/history.sqlite3",
        "/home/user/.local/share/fish/fish_history",
        "/home/user/.local/share/atuin/history.db",
        "/home/user/.local/share/atuin/history.db-shm",
        "/home/user/.local/share/atuin/history.db-wal",
        "/home/user/.python_history",
    ] {
        let output = run_file("Read", path);
        assert_eq!(
            blocked_reason(&output).expect("should block"),
            "Access to sensitive file blocked",
            "{path}"
        );
    }
}

#[test]
fn block_secrets_blocks_grep_atuin_directory() {
    let output = run_file("Grep", "/home/user/.local/share/atuin");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to sensitive file blocked"
    );
}

#[test]
fn block_secrets_allows_read_unrelated_history_files() {
    for path in [
        "src/history.rs",
        "docs/history.txt",
        "nushell/config.nu",
        "/home/user/.config/atuin/config.toml",
        "/home/user/dev/atuin/src/main.rs",
    ] {
        let output = run_file("Read", path);
        assert!(blocked_reason(&output).is_none(), "{path}");
    }
}

#[test]
fn block_secrets_allows_read_envrc() {
    let output = run_file("Read", ".envrc");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn block_secrets_allows_read_envrc_in_path() {
    let output = run_file("Read", "/home/user/project/.envrc");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn block_secrets_allows_read_encrypted_env() {
    let output = run_file("Read", ".env.production.yaml");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn block_secrets_allows_read_ssh_config() {
    let output = run_file("Read", "/home/user/.ssh/config");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn block_secrets_allows_grep_normal_file() {
    let output = run_file("Grep", "src/main.rs");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn block_secrets_allows_grep_path_field() {
    let mut input = HashMap::new();
    let key = "path".to_string();
    let path = "src/main.rs".to_string();
    let value = Value::String(path);
    input.insert(key, value);
    let grep_payload = payload("Grep", input);
    let output = run_pre_tool_use(&grep_payload);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn block_secrets_blocks_read_env_in_path() {
    let output = run_file("Read", "/app/.env");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to sensitive file blocked"
    );
}

#[test]
fn block_secrets_blocks_grep_ssh_key() {
    let output = run_file("Grep", "/home/user/.ssh/id_ed25519");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to sensitive file blocked"
    );
}

#[test]
fn block_secrets_blocks_read_etc_shadow() {
    let output = run_file("Read", "/etc/shadow");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to system secret file blocked"
    );
}

#[test]
fn block_secrets_blocks_read_etc_passwd() {
    let output = run_file("Read", "/etc/passwd");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to system secret file blocked"
    );
}

#[test]
fn block_secrets_blocks_grep_etc_gshadow() {
    let output = run_file("Grep", "/etc/gshadow");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to system secret file blocked"
    );
}

#[test]
fn block_secrets_blocks_read_etc_sudoers() {
    let output = run_file("Read", "/etc/sudoers");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to system secret file blocked"
    );
}

#[test]
fn block_secrets_blocks_read_etc_master_passwd() {
    let output = run_file("Read", "/etc/master.passwd");
    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Access to system secret file blocked"
    );
}

#[test]
fn commit_msg_check_allows_non_commit_command() {
    let output = run_bash("git push origin main");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_allows_no_edit_commit() {
    let output = run_bash("git commit --no-edit");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_allows_fixup_commit() {
    let output = run_bash("git commit --fixup abc123");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_allows_valid_feat_commit() {
    let output = run_bash(r#"git commit -m "[feat] Auth: add login""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_allows_valid_bugs_commit() {
    let output = run_bash(r#"git commit -m "[bugs] Fix: resolve crash""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_allows_valid_commit_with_body() {
    let output =
        run_bash(r#"git commit -m "[bugs] Fix: resolve crash" -m "Explain why the fix is needed""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_allows_valid_refa_commit() {
    let output = run_bash(r#"git commit -m "[refa] Config: update settings""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_allows_valid_deps_commit() {
    let output = run_bash(r#"git commit -m "[deps] Cargo: bump serde""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_allows_valid_tool_commit() {
    let output = run_bash(r#"git commit -m "[tool] CI: add check""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_allows_valid_single_quoted_commit() {
    let output = run_bash("git commit --only -m '[tool] Home: Add Pi coding agent'");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_blocks_unknown_tag() {
    let output = run_bash(r#"git commit -m "[chore] Area: description""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn commit_msg_check_blocks_missing_brackets() {
    let output = run_bash(r#"git commit -m "feat Area: description""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn commit_msg_check_blocks_missing_colon() {
    let output = run_bash(r#"git commit -m "[feat] Area description""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn commit_msg_check_blocks_missing_area() {
    let output = run_bash(r#"git commit -m "[feat] : description""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn commit_msg_check_blocks_missing_description_after_colon() {
    let output = run_bash(r#"git commit -m "[feat] Area:""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn commit_msg_check_allows_valid_heredoc_commit() {
    let output = run_bash("git commit -m \"$(cat <<'EOF'\n[feat] Auth: add login\nEOF\n)\"");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_blocks_heredoc_commit_with_co_authored_by() {
    let output = run_bash(
        "git commit -m \"$(cat <<'EOF'\n[feat] Auth: add login\n\nCo-Authored-By: Claude <noreply@anthropic.com>\nEOF\n)\"",
    );
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn commit_msg_check_allows_cd_prefix_commit() {
    let output = run_bash(
        r#"cd /Users/jruz/dev/dotfiles && git commit -m "[bugs] Deff: Update source hash""#,
    );
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn commit_msg_check_allows_multi_word_area() {
    let output = run_bash(r#"git commit -m "[feat] Auth module: add login""#);
    assert!(blocked_reason(&output).is_none());
}

fn message_options(message: &str) -> Vec<String> {
    SHORT_FLAG_PREFIXES
        .iter()
        .flat_map(|prefix| {
            [
                format!("-{prefix}m {message}"),
                format!("-{prefix}m{message}"),
            ]
        })
        .chain([
            format!("--message {message}"),
            format!("--message={message}"),
        ])
        .collect()
}

fn assert_commit_commands(commands: &[String], expected_reason: Option<&str>) {
    let mismatches = commands
        .iter()
        .filter_map(|command| {
            let output = run_bash(command);
            let reason = blocked_reason(&output);
            (reason.as_deref() != expected_reason).then(|| format!("{command}: {reason:?}"))
        })
        .collect::<Vec<_>>();
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn commit_msg_check_allows_all_short_flag_orders_and_message_attachment_forms() {
    let commands = [
        "'[feat] Auth: add login'",
        "\"[feat] Auth: add login\"",
        "\"$(cat <<'EOF'\n[feat] Auth: add login\nEOF\n)\"",
    ]
    .into_iter()
    .flat_map(message_options)
    .map(|options| format!("git commit {options}"))
    .collect::<Vec<_>>();
    assert_commit_commands(&commands, None);
}

#[test]
fn commit_msg_check_rejects_invalid_titles_for_all_message_options() {
    let commands = message_options("'invalid title'")
        .into_iter()
        .map(|options| format!("git commit {options}"))
        .collect::<Vec<_>>();
    let reason = Some("Invalid commit title, follow the Skill(commit) format");
    assert_commit_commands(&commands, reason);
}

#[test]
fn commit_msg_check_never_treats_message_values_as_skip_options() {
    let commands = ["'--no-edit'", "'--fixup'", "'--fixup=HEAD'"]
        .into_iter()
        .flat_map(message_options)
        .map(|options| format!("git commit {options}"))
        .collect::<Vec<_>>();
    let reason = Some("Invalid commit title, follow the Skill(commit) format");
    assert_commit_commands(&commands, reason);
}

#[test]
fn commit_msg_check_validates_first_message_in_every_pair_of_message_options() {
    let commands = message_options("'invalid title'")
        .into_iter()
        .flat_map(|first| {
            message_options("'[feat] Auth: add login'")
                .into_iter()
                .map(move |second| format!("git commit {first} {second}"))
        })
        .collect::<Vec<_>>();
    let reason = Some("Invalid commit title, follow the Skill(commit) format");
    assert_commit_commands(&commands, reason);
}

#[test]
fn commit_msg_check_allows_body_paragraphs_for_every_pair_of_message_options() {
    let commands = message_options("'[feat] Auth: add login'")
        .into_iter()
        .flat_map(|first| {
            message_options("'Explain why the fix is needed'")
                .into_iter()
                .map(move |second| format!("git commit {first} {second}"))
        })
        .collect::<Vec<_>>();
    assert_commit_commands(&commands, None);
}

#[test]
fn commit_msg_check_rejects_empty_messages_instead_of_using_later_titles() {
    let commands = message_options("''")
        .into_iter()
        .filter(|options| !options.ends_with("m''"))
        .map(|options| format!("git commit {options} -m '[feat] Auth: add login'"))
        .collect::<Vec<_>>();
    let reason = Some("Could not extract commit title");
    assert_commit_commands(&commands, reason);
}

#[test]
fn commit_msg_check_treats_attached_empty_message_as_bare_message_flag() {
    let commands = message_options("''")
        .into_iter()
        .filter(|options| options.ends_with("m''"))
        .map(|options| format!("git commit {options} -m '[feat] Auth: add login'"))
        .collect::<Vec<_>>();
    let reason = Some("Invalid commit title, follow the Skill(commit) format");
    assert_commit_commands(&commands, reason);
}

#[test]
fn commit_msg_check_rejects_missing_message_values_for_all_short_flag_orders() {
    let commands = SHORT_FLAG_PREFIXES
        .iter()
        .map(|prefix| format!("git commit -{prefix}m"))
        .chain(["git commit --message".to_string()])
        .collect::<Vec<_>>();
    let reason = Some("Could not extract commit title");
    assert_commit_commands(&commands, reason);
}

#[test]
fn commit_msg_check_treats_every_character_after_short_message_flag_as_message_text() {
    let commands = SHORT_FLAG_PREFIXES
        .iter()
        .flat_map(|prefix| {
            SHORT_FLAG_PREFIXES
                .iter()
                .filter(|suffix| !suffix.is_empty())
                .map(move |suffix| {
                    format!("git commit -{prefix}m{suffix} '[feat] Auth: add login'")
                })
        })
        .collect::<Vec<_>>();
    let reason = Some("Invalid commit title, follow the Skill(commit) format");
    assert_commit_commands(&commands, reason);
}

#[test]
fn commit_msg_check_preserves_explicit_skip_options_before_and_after_clustered_messages() {
    let commands = message_options("'invalid title'")
        .into_iter()
        .flat_map(|options| {
            ["--no-edit", "--fixup HEAD", "--fixup=HEAD"]
                .into_iter()
                .flat_map(move |skip| {
                    [
                        format!("git commit {skip} {options}"),
                        format!("git commit {options} {skip}"),
                    ]
                })
        })
        .collect::<Vec<_>>();
    assert_commit_commands(&commands, None);
}

#[test]
fn commit_msg_check_rejects_coauthors_in_all_message_forms_even_with_skip_options() {
    let commands = message_options(
        "'[feat] Auth: add login\n\nCo-Authored-By: Example <example@example.com>'",
    )
    .into_iter()
    .flat_map(|options| {
        ["", "--no-edit", "--fixup=HEAD"].map(|skip| format!("git commit {options} {skip}"))
    })
    .collect::<Vec<_>>();
    let reason = Some("Never include Co-Authored-By in commit messages");
    assert_commit_commands(&commands, reason);
}

#[test]
fn commit_msg_check_blocks_bare_commit_no_message() {
    let output = run_bash("git commit");
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn provider_fixture_allows_claude_grep_fixture_with_path_field() {
    let output = run_fixture("claude", "claude/pre_tool_use/grep_path.json");
    assert!(output.status.success());
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn provider_fixture_blocks_claude_read_fixture_for_sensitive_file() {
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

#[test]
fn shell_rules_blocks_git_pull_without_rebase() {
    crate::utils::shell_rules::blocks_git_pull_without_rebase("claude");
}

#[test]
fn shell_rules_allows_other_git_pull_commands_and_mentions() {
    crate::utils::shell_rules::allows_other_git_pull_commands_and_mentions("claude");
}

#[test]
fn shell_rules_allows_security_mentions_without_execution() {
    crate::utils::shell_rules::allows_security_mentions_without_execution("claude");
}

#[test]
fn shell_rules_blocks_security_executable_in_all_contexts() {
    crate::utils::shell_rules::blocks_security_executable_in_all_contexts("claude");
}

#[test]
fn shell_rules_allows_other_policy_mentions_as_text() {
    crate::utils::shell_rules::allows_other_policy_mentions_as_text("claude");
}

#[test]
fn shell_rules_blocks_decryption_with_shell_quoting_and_spacing() {
    crate::utils::shell_rules::blocks_decryption_with_shell_quoting_and_spacing("claude");
}

#[test]
fn shell_rules_allows_sops_recipes_metadata_encryption_and_literal_mentions() {
    crate::utils::shell_rules::allows_sops_recipes_metadata_encryption_and_literal_mentions(
        "claude",
    );
}

#[test]
fn shell_rules_preserves_quotes_and_escapes_in_nested_shell_commands() {
    crate::utils::shell_rules::preserves_quotes_and_escapes_in_nested_shell_commands("claude");
}

#[test]
fn shell_rules_blocks_gleam_run_after_wrappers_and_separators() {
    crate::utils::shell_rules::blocks_gleam_run_after_wrappers_and_separators("claude");
}

#[test]
fn shell_rules_keeps_file_exemptions_local_to_each_operand_and_command() {
    crate::utils::shell_rules::keeps_file_exemptions_local_to_each_operand_and_command("claude");
}

#[test]
fn shell_rules_allows_sensitive_path_mentions_as_text() {
    crate::utils::shell_rules::allows_sensitive_path_mentions_as_text("claude");
}

#[test]
fn shell_rules_blocks_system_secret_reads_after_unquoting() {
    crate::utils::shell_rules::blocks_system_secret_reads_after_unquoting("claude");
}

#[test]
fn shell_rules_blocks_docker_credentials_in_all_shell_contexts() {
    crate::utils::shell_rules::blocks_docker_credentials_in_all_shell_contexts("claude");
}

#[test]
fn shell_rules_allows_docker_path_mentions_and_unrelated_files() {
    crate::utils::shell_rules::allows_docker_path_mentions_and_unrelated_files("claude");
}

#[test]
fn shell_rules_blocks_all_plaintext_env_shell_variants() {
    crate::utils::shell_rules::blocks_all_plaintext_env_shell_variants("claude");
}

#[test]
fn shell_rules_allows_reading_and_writing_exact_env_example_files() {
    crate::utils::shell_rules::allows_reading_and_writing_exact_env_example_files("claude");
}

#[test]
fn shell_rules_allows_filename_only_metadata_checks() {
    crate::utils::shell_rules::allows_filename_only_metadata_checks("claude");
}

#[test]
fn shell_rules_metadata_checks_do_not_allow_sensitive_content_or_mutation() {
    crate::utils::shell_rules::metadata_checks_do_not_allow_sensitive_content_or_mutation("claude");
}

#[test]
fn shell_rules_allows_sed_substitution_text_without_treating_it_as_a_file() {
    crate::utils::shell_rules::allows_sed_substitution_text_without_treating_it_as_a_file("claude");
}

#[test]
fn shell_rules_sed_expression_exemptions_do_not_allow_sensitive_file_operands() {
    crate::utils::shell_rules::sed_expression_exemptions_do_not_allow_sensitive_file_operands(
        "claude",
    );
}

#[test]
fn shell_rules_allows_file_exceptions_in_shell_operands() {
    crate::utils::shell_rules::allows_file_exceptions_in_shell_operands("claude");
}

#[test]
fn shell_rules_rejects_cross_directory_exceptions_and_private_key_operands() {
    crate::utils::shell_rules::rejects_cross_directory_exceptions_and_private_key_operands(
        "claude",
    );
}

#[test]
fn shell_rules_blocks_environment_dump_commands_in_all_contexts() {
    crate::utils::shell_rules::blocks_environment_dump_commands_in_all_contexts("claude");
}

#[test]
fn shell_rules_blocks_direct_variable_output_without_blocking_variable_use() {
    crate::utils::shell_rules::blocks_direct_variable_output_without_blocking_variable_use(
        "claude",
    );
}

#[test]
fn shell_rules_parse_failures_explain_inspection_failure_without_echoing_input() {
    crate::utils::shell_rules::parse_failures_explain_inspection_failure_without_echoing_input(
        "claude",
    );
}

#[test]
fn shell_rules_sandbox_wrapper_path_denials_explain_the_bare_command_alternative() {
    crate::utils::shell_rules::sandbox_wrapper_path_denials_explain_the_bare_command_alternative(
        "claude",
    );
}

#[test]
fn shell_rules_allows_literal_bracket_predicates() {
    crate::utils::shell_rules::allows_literal_bracket_predicates("claude");
}

#[test]
fn shell_rules_bracket_predicates_do_not_allow_dynamic_executables() {
    crate::utils::shell_rules::bracket_predicates_do_not_allow_dynamic_executables("claude");
}

#[test]
fn shell_rules_allows_ssh_metadata_reads_and_edits_and_public_key_reads() {
    crate::utils::shell_rules::allows_ssh_metadata_reads_and_edits_and_public_key_reads("claude");
}

#[test]
fn shell_rules_ssh_metadata_exceptions_do_not_allow_private_keys_or_other_sensitive_files() {
    crate::utils::shell_rules::ssh_metadata_exceptions_do_not_allow_private_keys_or_other_sensitive_files("claude");
}

#[test]
fn shell_rules_allows_ordinary_variable_output() {
    crate::utils::shell_rules::allows_ordinary_variable_output("claude");
}

#[test]
fn shell_rules_allows_plain_lowercase_variable_output() {
    crate::utils::shell_rules::allows_plain_lowercase_variable_output("claude");
}

#[test]
fn shell_rules_sensitive_names_and_unsupported_expansions_remain_blocked() {
    crate::utils::shell_rules::sensitive_names_and_unsupported_expansions_remain_blocked("claude");
}

#[test]
fn shell_rules_allows_printenv_with_explicit_ordinary_names() {
    crate::utils::shell_rules::allows_printenv_with_explicit_ordinary_names("claude");
}

#[test]
fn shell_rules_blocks_printenv_sensitive_names_dynamic_operands_and_dumps() {
    crate::utils::shell_rules::blocks_printenv_sensitive_names_dynamic_operands_and_dumps("claude");
}

#[test]
fn shell_rules_allows_environment_wrappers_and_literal_mentions() {
    crate::utils::shell_rules::allows_environment_wrappers_and_literal_mentions("claude");
}

#[test]
fn shell_rules_fd_lists_sensitive_paths_without_reading_them() {
    crate::utils::shell_rules::fd_lists_sensitive_paths_without_reading_them("claude");
}

#[test]
fn shell_rules_env_split_strings_preserve_trailing_arguments() {
    crate::utils::shell_rules::env_split_strings_preserve_trailing_arguments("claude");
}

#[test]
fn shell_rules_env_split_strings_visit_commands_after_quoted_assignments() {
    crate::utils::shell_rules::env_split_strings_visit_commands_after_quoted_assignments("claude");
}

#[test]
fn shell_rules_blocks_shell_tracing_that_can_print_secret_arguments() {
    crate::utils::shell_rules::blocks_shell_tracing_that_can_print_secret_arguments("claude");
}

#[test]
fn shell_rules_blocks_actual_hook_skipping_assignments() {
    crate::utils::shell_rules::blocks_actual_hook_skipping_assignments("claude");
}

#[test]
fn shell_rules_validates_actual_commit_messages_not_shell_text() {
    crate::utils::shell_rules::validates_actual_commit_messages_not_shell_text("claude");
}

#[test]
fn shell_rules_validates_literal_heredoc_messages_without_executing_shell() {
    crate::utils::shell_rules::validates_literal_heredoc_messages_without_executing_shell("claude");
}

#[test]
fn shell_rules_validates_literal_commit_messages_from_stdin() {
    crate::utils::shell_rules::validates_literal_commit_messages_from_stdin("claude");
}

#[test]
fn shell_rules_requires_known_literal_stdin_for_commit_messages() {
    crate::utils::shell_rules::requires_known_literal_stdin_for_commit_messages("claude");
}

#[test]
fn shell_rules_blocks_coauthor_only_in_actual_commit_arguments() {
    crate::utils::shell_rules::blocks_coauthor_only_in_actual_commit_arguments("claude");
}

#[test]
fn sandbox_policy_dependency_executables_require_sandbox_in_common_invocation_forms() {
    crate::utils::sandbox_policy::dependency_executables_require_sandbox_in_common_invocation_forms(
        "claude",
    );
}

#[test]
fn sandbox_policy_dependency_execution_subcommands_and_flags_require_sandbox() {
    crate::utils::sandbox_policy::dependency_execution_subcommands_and_flags_require_sandbox(
        "claude",
    );
}

#[test]
fn sandbox_policy_dependency_executables_are_checked_in_shell_pipelines_and_substitutions() {
    crate::utils::sandbox_policy::dependency_executables_are_checked_in_shell_pipelines_and_substitutions("claude");
}

#[test]
fn sandbox_policy_dependency_names_in_text_and_unrelated_executables_are_allowed() {
    crate::utils::sandbox_policy::dependency_names_in_text_and_unrelated_executables_are_allowed(
        "claude",
    );
}

#[test]
fn sandbox_policy_local_sandbox_requirements_do_not_apply_to_remote_or_container_execution() {
    crate::utils::sandbox_policy::local_sandbox_requirements_do_not_apply_to_remote_or_container_execution("claude");
}

#[test]
fn sandbox_policy_shells_and_generic_launchers_do_not_require_sandbox() {
    crate::utils::sandbox_policy::shells_and_generic_launchers_do_not_require_sandbox("claude");
}

#[test]
fn file_policy_blocks_plaintext_env_variants_for_file_tools() {
    crate::utils::file_policy::blocks_plaintext_env_variants_for_file_tools("claude");
}

#[test]
fn file_policy_allows_only_file_exceptions_inside_sensitive_paths() {
    crate::utils::file_policy::allows_only_file_exceptions_inside_sensitive_paths("claude");
}

#[test]
fn file_policy_file_exceptions_do_not_allow_other_sensitive_operands() {
    crate::utils::file_policy::file_exceptions_do_not_allow_other_sensitive_operands("claude");
}

#[test]
fn file_policy_allows_ordinary_files_without_secret_name_false_positives() {
    crate::utils::file_policy::allows_ordinary_files_without_secret_name_false_positives("claude");
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
