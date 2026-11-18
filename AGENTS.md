# agent-tool-check

Rust CLI that runs the PreToolUse hook for Claude Code, Codex, and Pi.

## Release

- `just release bump <version>` updates Cargo manifests and commits the version bump.
- Push the bump to `main`; CI handles tagging and publishing.
- `just release delete <tag>` deletes a published release and its tag.

## Hook

`~/dev/projects/agent-config/hooks/on_tool` invokes the binary for all agents.

## Tooling

`cargo` and `bun` are only available inside the project sandbox. Do not run `cargo ...` or `bun ...` directly from the shell.

Use the `just` recipes instead so they execute through the repo's sandbox wrapper:

- `just check`
- `just test`
- `just fix`
- `just run ...`
- `just rust::build-host`
- `just rust::check`
- `just rust::test`

`just setup` installs TypeScript dependencies. The sandbox builds runtime images on first use and reuses them until their Dockerfiles or runtime pins change.

## Reviewing Historical Denials

```sh
just denials-list --since-days 2
```

- Filter with `--agent claude|codex|pi`.
- Limit results with `--limit N`.
- Use `--json` for metadata without commands.
- Read `denial.log` without agent transcripts by default.
- Use `--transcripts` for historical agent logs.

## Architecture

- `src/main.rs`: CLI entrypoint.
- `src/commands/`: thin command modules.
- `src/core/`: hook orchestration and policies.
- `src/libs/`: self-contained protocol response builders.
- `src/utils/`: generic output and logging utilities.
- `tests/main.rs`: integration test entrypoint.
- `tests/commands/`: one test file per CLI command.
- `tests/utils/`: shared subprocess assertions.
- `tests/fixtures/`: provider payloads.

### Layer Rules

- `main` calls `commands` and `utils`.
- `commands` calls `core` and `utils`.
- `core` calls `libs` and `utils`.
- `libs` calls `utils`.
- `utils` has no sibling-module dependencies.

### Architecture Checks

Run `just rust::architecture-check`.

## Conventions

- filter modules return raw data, the orchestrator handles blocking logic
- provider-specific wire parsing stays at the event edge; shared modules hold policy logic
- all Bash providers use one shell visitor; policies inspect parsed invocations, assignments, and redirects rather than raw command strings
- `src/libs/response.rs` builds the JSON hook callers expect
- `main.rs` owns all printing (stdout/stderr) and error logging to file
- each module has its own thiserror enum
- event modules use `self_named_module_files` style

## Logging

- Errors: `~/.local/state/agent-tool-check/errors.log`.
- Denials: `~/.local/state/agent-tool-check/denial.log`.
- Both logs use JSONL.
- Denials include redacted commands and debugging context.
- `XDG_STATE_HOME` controls the state directory.
- Read errors with `just logs::errors-read`.
- Read denials with `just logs::denials-read`.
- Clear errors with `just logs::errors-clear`.
- Clear denials with `just logs::denials-clear`.
