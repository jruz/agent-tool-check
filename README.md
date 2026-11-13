# agent-tool-check

Blocks risky agent tool calls before they run.

Supports Claude Code, Codex, and Pi.

- blocks reads of sensitive files such as `.env` and private keys
- blocks environment variable output and shell tracing
- requires package managers and build tools to be prefixed with `sandbox`, a command you provide that isolates them (for example in Docker)
- validates commit titles match `[feat|bugs|refa|deps|tool] Area: summary`
- rejects `Co-Authored-By` trailers in commit messages

## Install with Nix

Requires Nix with flakes enabled.

```sh
nix profile add github:jruz/agent-tool-check
```

Installs a published binary without compiling Rust. Supports Linux and macOS on x86_64 and aarch64.

Verify the installation:

```sh
agent-tool-check --version
```

Upgrade:

```sh
nix profile upgrade agent-tool-check
```

Run without installing:

```sh
nix run github:jruz/agent-tool-check -- --help
```

Install from source instead:

```sh
nix profile add github:jruz/agent-tool-check#source
```

## Install without Nix

Download the binary for your platform from [releases](https://github.com/jruz/agent-tool-check/releases) and put it on your `PATH`.

## Configure

Claude Code, `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash|Read|Grep",
        "hooks": [{ "type": "command", "command": "agent-tool-check claude" }]
      }
    ]
  }
}
```

Codex, `~/.codex/hooks.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": "agent-tool-check codex" }]
      }
    ]
  }
}
```

Pi: run `agent-tool-check pi` from a `tool_call` extension, passing the event as JSON on stdin.

## License

Licensed under the [European Union Public Licence v1.2](LICENSE).
