# Transcript denials

```sh
just bun-install
just denials-list --since-days 2 --agent pi
```

- Use `claude`, `codex`, or `pi` to filter agents.
- Default output is JSON keyed by each complete hook error converted to lowercase kebab-case.
- Each key contains occurrence objects with the triggering command; error text is not duplicated.
- Details include project, agent, and timestamp.
- Multiline commands retain newlines in their JSON string.
- Common credential assignments and flags are redacted; other sensitive command text may remain.
- Use `--json` for the full metadata report without commands.
- In JSON, Pi errors appear in `failedToolCalls`, independently limited by `--limit`.
- All agents match complete tool-error text against our shared hook-message catalog in `hook-errors.ts`.
- Claude and Codex hook wrappers are removed before matching.
- Codex supports string results and `input_text`/`text` blocks with `Script error:` envelopes.
- Codex `exec` commands display the original script source without evaluating it.
- Script success output containing hook messages is not treated as a denial.
- Unknown or changed messages remain unclassified.
- Matches identify our messages rather than every failed tool or hook.
- `failedToolCalls` includes all Pi errors, including matching denials.
- Raw tool outputs are omitted.
- `--since-days` is required and selects the last N × 24 hours using a positive integer.
- `--limit` defaults to `100`.
- `--agent` defaults to `all`.
- `--limit` selects the newest occurrences globally before grouping.
- The full metadata report counts all matching records.
- JSON transcript paths and line numbers identify the original invocations.
- Malformed lines produce warnings without discarding the remaining transcript.
- Historical denials reflect the rules installed at the time.

## SOPS audit

- `sops.summary` counts every matching shell-tool call within the selected period.
- `sops.calls` lists the newest candidates using the same `--limit` independently.
- `sops-token` matches standalone SOPS tokens including nested commands and quoted mentions.
- `recipe-reference` matches names such as `sops-decrypt-dev` without a standalone token.
- `decryptReference` flags decrypt text anywhere in the command; it is not proof of invocation.
- Outcomes distinguish hook denials from tool results, Claude/Pi tool errors, and missing results.
- `tool-result` does not prove successful decryption; Codex outputs are not interpreted as exit statuses.
- Commands and tool outputs are never included in the SOPS audit.
- Text matching intentionally includes searches and recipe definitions for manual review.
- Dynamically constructed executable names and indirect script execution are not detected.

The TypeScript script reads transcripts directly. The sandbox mounts `~/.claude/projects` and `~/.codex/sessions` read-only by default. It also mounts `~/.pi/agent/sessions` read-only. The sandbox only runs Bun, Node, pnpm, and Rust tools in Docker. Other commands are rejected. Other agent configuration remains hidden.

Reload the development shell after changing `flake.nix`. Unavailable selected directories produce `sourceFailures` and a nonzero exit status.
