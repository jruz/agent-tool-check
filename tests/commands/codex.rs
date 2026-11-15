use std::io::Write as _;
use std::process::Command;

use crate::utils::fixture_pre_tool_use as helpers;

use helpers::{blocked_reason, run_fixture, stdout_json};
use serde_json::Value;

#[test]
fn ager_blocks_plaintext() {
    crate::utils::ager_policy::blocks_plaintext("codex");
}

#[test]
fn ager_allows_ciphertext_and_unlisted() {
    crate::utils::ager_policy::allows_ciphertext_and_unlisted("codex");
}

#[test]
fn ager_only_uses_current_project() {
    crate::utils::ager_policy::only_uses_current_project("codex");
}

#[test]
fn ager_rejects_invalid_manifests() {
    crate::utils::ager_policy::rejects_invalid_manifests("codex");
}

#[test]
fn blocks_codex_bash_env_file() {
    let output = run_fixture("codex", "codex/pre_tool_use/bash_env.json");
    let json = stdout_json(&output).expect("should emit json");
    let hook_specific = json
        .get("hookSpecificOutput")
        .and_then(Value::as_object)
        .expect("should emit hookSpecificOutput");

    assert_eq!(
        blocked_reason(&output).expect("should block"),
        "Never access .env files directly"
    );
    assert_eq!(
        hook_specific.get("hookEventName").and_then(Value::as_str),
        Some("PreToolUse")
    );
    assert_eq!(
        hook_specific
            .get("permissionDecision")
            .and_then(Value::as_str),
        Some("deny")
    );
    assert_eq!(
        hook_specific
            .get("permissionDecisionReason")
            .and_then(Value::as_str),
        Some("Never access .env files directly")
    );
    assert!(json.get("decision").is_none());
}

#[test]
fn allows_safe_codex_bash() {
    let output = run_fixture("codex", "codex/pre_tool_use/git_status.json");
    assert!(blocked_reason(&output).is_none());
}

#[test]
fn ignores_non_bash_codex_tool() {
    let output = run_fixture("codex", "codex/pre_tool_use/read_env.json");
    assert!(blocked_reason(&output).is_none());
    assert!(stdout_json(&output).is_none());
}

#[test]
fn shell_rules_blocks_git_pull_without_rebase() {
    crate::utils::shell_rules::blocks_git_pull_without_rebase("codex");
}

#[test]
fn shell_rules_allows_other_git_pull_commands_and_mentions() {
    crate::utils::shell_rules::allows_other_git_pull_commands_and_mentions("codex");
}

#[test]
fn shell_rules_allows_security_mentions_without_execution() {
    crate::utils::shell_rules::allows_security_mentions_without_execution("codex");
}

#[test]
fn shell_rules_blocks_security_executable_in_all_contexts() {
    crate::utils::shell_rules::blocks_security_executable_in_all_contexts("codex");
}

#[test]
fn shell_rules_allows_other_policy_mentions_as_text() {
    crate::utils::shell_rules::allows_other_policy_mentions_as_text("codex");
}

#[test]
fn shell_rules_blocks_decryption_with_shell_quoting_and_spacing() {
    crate::utils::shell_rules::blocks_decryption_with_shell_quoting_and_spacing("codex");
}

#[test]
fn shell_rules_allows_sops_recipes_metadata_encryption_and_literal_mentions() {
    crate::utils::shell_rules::allows_sops_recipes_metadata_encryption_and_literal_mentions(
        "codex",
    );
}

#[test]
fn shell_rules_preserves_quotes_and_escapes_in_nested_shell_commands() {
    crate::utils::shell_rules::preserves_quotes_and_escapes_in_nested_shell_commands("codex");
}

#[test]
fn shell_rules_blocks_gleam_run_after_wrappers_and_separators() {
    crate::utils::shell_rules::blocks_gleam_run_after_wrappers_and_separators("codex");
}

#[test]
fn shell_rules_keeps_file_exemptions_local_to_each_operand_and_command() {
    crate::utils::shell_rules::keeps_file_exemptions_local_to_each_operand_and_command("codex");
}

#[test]
fn shell_rules_allows_sensitive_path_mentions_as_text() {
    crate::utils::shell_rules::allows_sensitive_path_mentions_as_text("codex");
}

#[test]
fn shell_rules_blocks_system_secret_reads_after_unquoting() {
    crate::utils::shell_rules::blocks_system_secret_reads_after_unquoting("codex");
}

#[test]
fn shell_rules_blocks_docker_credentials_in_all_shell_contexts() {
    crate::utils::shell_rules::blocks_docker_credentials_in_all_shell_contexts("codex");
}

#[test]
fn shell_rules_allows_docker_path_mentions_and_unrelated_files() {
    crate::utils::shell_rules::allows_docker_path_mentions_and_unrelated_files("codex");
}

#[test]
fn shell_rules_blocks_all_plaintext_env_shell_variants() {
    crate::utils::shell_rules::blocks_all_plaintext_env_shell_variants("codex");
}

#[test]
fn shell_rules_allows_reading_and_writing_exact_env_example_files() {
    crate::utils::shell_rules::allows_reading_and_writing_exact_env_example_files("codex");
}

#[test]
fn shell_rules_allows_filename_only_metadata_checks() {
    crate::utils::shell_rules::allows_filename_only_metadata_checks("codex");
}

#[test]
fn shell_rules_metadata_checks_do_not_allow_sensitive_content_or_mutation() {
    crate::utils::shell_rules::metadata_checks_do_not_allow_sensitive_content_or_mutation("codex");
}

#[test]
fn shell_rules_allows_sed_substitution_text_without_treating_it_as_a_file() {
    crate::utils::shell_rules::allows_sed_substitution_text_without_treating_it_as_a_file("codex");
}

#[test]
fn shell_rules_sed_expression_exemptions_do_not_allow_sensitive_file_operands() {
    crate::utils::shell_rules::sed_expression_exemptions_do_not_allow_sensitive_file_operands(
        "codex",
    );
}

#[test]
fn shell_rules_allows_file_exceptions_in_shell_operands() {
    crate::utils::shell_rules::allows_file_exceptions_in_shell_operands("codex");
}

#[test]
fn shell_rules_rejects_cross_directory_exceptions_and_private_key_operands() {
    crate::utils::shell_rules::rejects_cross_directory_exceptions_and_private_key_operands("codex");
}

#[test]
fn shell_rules_blocks_environment_dump_commands_in_all_contexts() {
    crate::utils::shell_rules::blocks_environment_dump_commands_in_all_contexts("codex");
}

#[test]
fn shell_rules_blocks_direct_variable_output_without_blocking_variable_use() {
    crate::utils::shell_rules::blocks_direct_variable_output_without_blocking_variable_use("codex");
}

#[test]
fn shell_rules_parse_failures_explain_inspection_failure_without_echoing_input() {
    crate::utils::shell_rules::parse_failures_explain_inspection_failure_without_echoing_input(
        "codex",
    );
}

#[test]
fn shell_rules_sandbox_wrapper_path_denials_explain_the_bare_command_alternative() {
    crate::utils::shell_rules::sandbox_wrapper_path_denials_explain_the_bare_command_alternative(
        "codex",
    );
}

#[test]
fn shell_rules_allows_literal_bracket_predicates() {
    crate::utils::shell_rules::allows_literal_bracket_predicates("codex");
}

#[test]
fn shell_rules_bracket_predicates_do_not_allow_dynamic_executables() {
    crate::utils::shell_rules::bracket_predicates_do_not_allow_dynamic_executables("codex");
}

#[test]
fn shell_rules_allows_ssh_metadata_reads_and_edits_and_public_key_reads() {
    crate::utils::shell_rules::allows_ssh_metadata_reads_and_edits_and_public_key_reads("codex");
}

#[test]
fn shell_rules_ssh_metadata_exceptions_do_not_allow_private_keys_or_other_sensitive_files() {
    crate::utils::shell_rules::ssh_metadata_exceptions_do_not_allow_private_keys_or_other_sensitive_files("codex");
}

#[test]
fn shell_rules_allows_ordinary_variable_output() {
    crate::utils::shell_rules::allows_ordinary_variable_output("codex");
}

#[test]
fn shell_rules_allows_plain_lowercase_variable_output() {
    crate::utils::shell_rules::allows_plain_lowercase_variable_output("codex");
}

#[test]
fn shell_rules_sensitive_names_and_unsupported_expansions_remain_blocked() {
    crate::utils::shell_rules::sensitive_names_and_unsupported_expansions_remain_blocked("codex");
}

#[test]
fn shell_rules_allows_printenv_with_explicit_ordinary_names() {
    crate::utils::shell_rules::allows_printenv_with_explicit_ordinary_names("codex");
}

#[test]
fn shell_rules_blocks_printenv_sensitive_names_dynamic_operands_and_dumps() {
    crate::utils::shell_rules::blocks_printenv_sensitive_names_dynamic_operands_and_dumps("codex");
}

#[test]
fn shell_rules_allows_environment_wrappers_and_literal_mentions() {
    crate::utils::shell_rules::allows_environment_wrappers_and_literal_mentions("codex");
}

#[test]
fn shell_rules_fd_lists_sensitive_paths_without_reading_them() {
    crate::utils::shell_rules::fd_lists_sensitive_paths_without_reading_them("codex");
}

#[test]
fn shell_rules_env_split_strings_preserve_trailing_arguments() {
    crate::utils::shell_rules::env_split_strings_preserve_trailing_arguments("codex");
}

#[test]
fn shell_rules_env_split_strings_visit_commands_after_quoted_assignments() {
    crate::utils::shell_rules::env_split_strings_visit_commands_after_quoted_assignments("codex");
}

#[test]
fn shell_rules_blocks_shell_tracing_that_can_print_secret_arguments() {
    crate::utils::shell_rules::blocks_shell_tracing_that_can_print_secret_arguments("codex");
}

#[test]
fn shell_rules_blocks_actual_hook_skipping_assignments() {
    crate::utils::shell_rules::blocks_actual_hook_skipping_assignments("codex");
}

#[test]
fn shell_rules_validates_actual_commit_messages_not_shell_text() {
    crate::utils::shell_rules::validates_actual_commit_messages_not_shell_text("codex");
}

#[test]
fn shell_rules_validates_literal_heredoc_messages_without_executing_shell() {
    crate::utils::shell_rules::validates_literal_heredoc_messages_without_executing_shell("codex");
}

#[test]
fn shell_rules_validates_literal_commit_messages_from_stdin() {
    crate::utils::shell_rules::validates_literal_commit_messages_from_stdin("codex");
}

#[test]
fn shell_rules_requires_known_literal_stdin_for_commit_messages() {
    crate::utils::shell_rules::requires_known_literal_stdin_for_commit_messages("codex");
}

#[test]
fn shell_rules_blocks_coauthor_only_in_actual_commit_arguments() {
    crate::utils::shell_rules::blocks_coauthor_only_in_actual_commit_arguments("codex");
}

#[test]
fn sandbox_policy_dependency_executables_require_sandbox_in_common_invocation_forms() {
    crate::utils::sandbox_policy::dependency_executables_require_sandbox_in_common_invocation_forms(
        "codex",
    );
}

#[test]
fn sandbox_policy_dependency_execution_subcommands_and_flags_require_sandbox() {
    crate::utils::sandbox_policy::dependency_execution_subcommands_and_flags_require_sandbox(
        "codex",
    );
}

#[test]
fn sandbox_policy_dependency_executables_are_checked_in_shell_pipelines_and_substitutions() {
    crate::utils::sandbox_policy::dependency_executables_are_checked_in_shell_pipelines_and_substitutions("codex");
}

#[test]
fn sandbox_policy_dependency_names_in_text_and_unrelated_executables_are_allowed() {
    crate::utils::sandbox_policy::dependency_names_in_text_and_unrelated_executables_are_allowed(
        "codex",
    );
}

#[test]
fn sandbox_policy_local_sandbox_requirements_do_not_apply_to_remote_or_container_execution() {
    crate::utils::sandbox_policy::local_sandbox_requirements_do_not_apply_to_remote_or_container_execution("codex");
}

#[test]
fn sandbox_policy_shells_and_generic_launchers_do_not_require_sandbox() {
    crate::utils::sandbox_policy::shells_and_generic_launchers_do_not_require_sandbox("codex");
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
