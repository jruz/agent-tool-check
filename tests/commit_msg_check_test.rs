#[path = "support/pre_tool_use.rs"]
mod helpers;

use std::collections::HashMap;
use std::process::Output;

use helpers::{blocked_reason, payload, run_pre_tool_use};
use serde_json::Value;

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
fn allows_non_commit_command() {
    let output = run_bash("git push origin main");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn allows_no_edit_commit() {
    let output = run_bash("git commit --no-edit");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn allows_fixup_commit() {
    let output = run_bash("git commit --fixup abc123");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn allows_valid_feat_commit() {
    let output = run_bash(r#"git commit -m "[feat] Auth: add login""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn allows_valid_bugs_commit() {
    let output = run_bash(r#"git commit -m "[bugs] Fix: resolve crash""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn allows_valid_commit_with_body() {
    let output =
        run_bash(r#"git commit -m "[bugs] Fix: resolve crash" -m "Explain why the fix is needed""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn allows_valid_refa_commit() {
    let output = run_bash(r#"git commit -m "[refa] Config: update settings""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn allows_valid_deps_commit() {
    let output = run_bash(r#"git commit -m "[deps] Cargo: bump serde""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn allows_valid_tool_commit() {
    let output = run_bash(r#"git commit -m "[tool] CI: add check""#);
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn allows_valid_single_quoted_commit() {
    let output = run_bash("git commit --only -m '[tool] Home: Add Pi coding agent'");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn blocks_unknown_tag() {
    let output = run_bash(r#"git commit -m "[chore] Area: description""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn blocks_missing_brackets() {
    let output = run_bash(r#"git commit -m "feat Area: description""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn blocks_missing_colon() {
    let output = run_bash(r#"git commit -m "[feat] Area description""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn blocks_missing_area() {
    let output = run_bash(r#"git commit -m "[feat] : description""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn blocks_missing_description_after_colon() {
    let output = run_bash(r#"git commit -m "[feat] Area:""#);
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn allows_valid_heredoc_commit() {
    let output = run_bash("git commit -m \"$(cat <<'EOF'\n[feat] Auth: add login\nEOF\n)\"");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn blocks_heredoc_commit_with_co_authored_by() {
    let output = run_bash(
        "git commit -m \"$(cat <<'EOF'\n[feat] Auth: add login\n\nCo-Authored-By: Claude <noreply@anthropic.com>\nEOF\n)\"",
    );
    assert!(blocked_reason(&output).is_some());
}

#[test]
fn allows_cd_prefix_commit() {
    let output = run_bash(
        r#"cd /Users/jruz/dev/dotfiles && git commit -m "[bugs] Deff: Update source hash""#,
    );
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn allows_multi_word_area() {
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
fn allows_all_short_flag_orders_and_message_attachment_forms() {
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
fn rejects_invalid_titles_for_all_message_options() {
    let commands = message_options("'invalid title'")
        .into_iter()
        .map(|options| format!("git commit {options}"))
        .collect::<Vec<_>>();
    let reason = Some("Invalid commit title, follow the Skill(commit) format");
    assert_commit_commands(&commands, reason);
}

#[test]
fn never_treats_message_values_as_skip_options() {
    let commands = ["'--no-edit'", "'--fixup'", "'--fixup=HEAD'"]
        .into_iter()
        .flat_map(message_options)
        .map(|options| format!("git commit {options}"))
        .collect::<Vec<_>>();
    let reason = Some("Invalid commit title, follow the Skill(commit) format");
    assert_commit_commands(&commands, reason);
}

#[test]
fn validates_first_message_in_every_pair_of_message_options() {
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
fn allows_body_paragraphs_for_every_pair_of_message_options() {
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
fn rejects_empty_messages_instead_of_using_later_titles() {
    let commands = message_options("''")
        .into_iter()
        .filter(|options| !options.ends_with("m''"))
        .map(|options| format!("git commit {options} -m '[feat] Auth: add login'"))
        .collect::<Vec<_>>();
    let reason = Some("Could not extract commit title");
    assert_commit_commands(&commands, reason);
}

#[test]
fn treats_attached_empty_message_as_bare_message_flag() {
    let commands = message_options("''")
        .into_iter()
        .filter(|options| options.ends_with("m''"))
        .map(|options| format!("git commit {options} -m '[feat] Auth: add login'"))
        .collect::<Vec<_>>();
    let reason = Some("Invalid commit title, follow the Skill(commit) format");
    assert_commit_commands(&commands, reason);
}

#[test]
fn rejects_missing_message_values_for_all_short_flag_orders() {
    let commands = SHORT_FLAG_PREFIXES
        .iter()
        .map(|prefix| format!("git commit -{prefix}m"))
        .chain(["git commit --message".to_string()])
        .collect::<Vec<_>>();
    let reason = Some("Could not extract commit title");
    assert_commit_commands(&commands, reason);
}

#[test]
fn treats_every_character_after_short_message_flag_as_message_text() {
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
fn preserves_explicit_skip_options_before_and_after_clustered_messages() {
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
fn rejects_coauthors_in_all_message_forms_even_with_skip_options() {
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
fn blocks_bare_commit_no_message() {
    let output = run_bash("git commit");
    assert!(blocked_reason(&output).is_some());
}
