# agent-tool-check

Rust CLI that runs the PreToolUse hook for Claude Code, Codex, and Pi.

## Usage

```sh
agent-tool-check claude # Claude PreToolUse hook (reads stdin)
agent-tool-check codex  # Codex PreToolUse hook (reads stdin)
agent-tool-check pi     # Pi PreToolUse hook (reads stdin)
```

Configure each agent's PreToolUse hook to pass the matching agent.

## Release

- `just release::run bump <version>` updates Cargo manifests and commits the version bump.
- Push the bump to `main`; CI handles tagging and publishing.
- `just release::run delete <tag>` deletes a published release and its tag.

## Hook Files

- `~/.claude/settings.json`
- `~/.codex/hooks.json`
- `~/.pi/agent/settings.json`

## Tooling

`cargo` and `bun` are only available inside the project sandbox. Do not run `cargo ...` or `bun ...` directly from the shell.

Use the `just` recipes instead so they execute through the repo's sandbox wrapper:

- `just check`
- `just test`
- `just fix`
- `just run ...`
- `just rust::build`
- `just rust::check`
- `just rust::test`
- `just release::run upload <tag>`
- `just release::run delete <tag>`

`just setup` installs TypeScript dependencies. The sandbox builds runtime images on first use and reuses them until their Dockerfiles or runtime pins change.

## Reviewing Historical Denials

Use the TypeScript report to find our known hook-error messages in Claude, Codex, and Pi tool results:

Last two days across all agents:

```sh
just denials-list --since-days 2
```

Older Codex denials:

```sh
just denials-list --since-days 365 --agent codex --limit 1000
```

- `--since-days`: required positive integer selecting the last N × 24 hours.
- `--limit`: positive integer limiting the newest denials listed; defaults to `100`.
- `--agent`: `all`, `claude`, `codex`, or `pi`; defaults to `all`.

- Default output groups occurrence details by the complete error converted to lowercase kebab-case.
- Details include the triggering command.
- Common credential forms are redacted; other sensitive command text may remain.
- The limit selects newest occurrences before grouping.
- `--json` returns the full metadata report without commands.
- Only known hook-error strings are matched; unknown historical messages remain unmatched.

The project configures read-only transcript mounts in `sandbox.json`. The sandbox only runs Bun, Node, pnpm, and Rust tools in Docker. Other commands are rejected.

See `scripts/denials-list/README.md` for report behavior.

## Architecture

```
src/
  main.rs                # clap CLI, handles output and error logging
  pre_tool_use.rs        # PreToolUse hook orchestrator
  pre_tool_use/
    claude.rs            # Claude-specific payload parsing and response rendering
    codex.rs             # Codex-specific payload parsing and response rendering
    pi.rs                # Pi-specific payload parsing and response rendering
    filters.rs           # shared PreToolUse filter exports
    filters/
      bash.rs            # shared shell policy orchestrator
      bash/
        curl.rs          # inspect curl file inputs and outputs
        environment.rs   # block environment output and shell tracing
        paths.rs         # inspect command file operands and redirects
        rules.rs         # executable, flag, and assignment policies
        sandboxed_executable.rs # enforce local sandbox requirements
      commit_msg.rs      # validate parsed git commit messages
      secrets.rs         # shared sensitive-file policy and read exceptions
      shell.rs           # shared shell parser, visitor, and execution context
  shared/
    agent.rs             # shared clap value enum for the agent argument
    env.rs               # shared environment path helpers
    mod.rs               # shared module exports
    response.rs          # hook JSON response builders
    logger.rs            # structured JSONL file logger
```

## Conventions

- filter modules return raw data, the orchestrator handles blocking logic
- provider-specific wire parsing stays at the event edge; shared modules hold policy logic
- all Bash providers use one shell visitor; policies inspect parsed invocations, assignments, and redirects rather than raw command strings
- local sandbox requirements target dependency-executing runtimes and package/build tools, not shells or generic launchers
- sandbox exemptions apply only to sandbox requirements, not command or file restrictions
- SSH and Docker commands retain remote/container execution context; local sandbox requirements are not imposed on their executables
- `shared/response` builds the JSON hook callers expect
- `main.rs` owns all printing (stdout/stderr) and error logging to file
- each module has its own thiserror enum
- event modules use `self_named_module_files` style, `shared/` uses `mod.rs`

## SSH File Access

- Read and edit `.ssh/known_hosts`.
- Read and edit `.ssh/authorized_keys`.
- Read public-key files with the `.pub` suffix.
- Private-key access remains blocked.
- An allowed operand does not exempt another sensitive operand.

## Variable Output Policy

- Output blocks underscore-separated name parts `TOKEN`, `KEY`, `SECRET`, `PASSWORD`, and `PASSPHRASE` case-insensitively.
- Ordinary named-variable output is permitted.
- `printenv` permits explicit non-sensitive names with optional leading `-0`, `--null`, or `--`.
- Dynamic `printenv` operands remain blocked.
- Modified expansions also check embedded variable expansions.
- Assignments and non-output variable use are not blocked by this rule.
- Indirect expansions check only the written name without resolving its target.
- CamelCase names such as `apiToken` do not match sensitive-name parts.
- This guards against accidental disclosure rather than adversarial bypasses.
- Full environment dumps remain blocked.
- Shell tracing remains blocked.
- Arithmetic output remains blocked.
- Tilde output remains blocked.
- Positional output remains blocked.
- Special parameters other than exit status remain blocked.
- Variable-name enumeration and array-key enumeration remain blocked.

## Hook Protocol

Claude Code hooks communicate via JSON on stdout:

- **block**: `{"decision": "block", "reason": "..."}` — stops Claude and shows the reason
- no output — hook passes silently

Codex deny responses use `hookSpecificOutput.permissionDecision = "deny"`.
Codex currently only handles `Bash` events.

Pi accepts Pi's native `toolName` and `input` fields. Blocked
responses use `{ "block": true, "reason": "..." }`.

## Logging

Errors are logged as JSONL to `~/.local/state/agent-tool-check/hooks.log` from `main.rs`. If logging fails, errors fall back to stderr. Respects `XDG_STATE_HOME` if set.
