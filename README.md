# agent-tool-check

Hooks for Claude Code, Codex, and Pi.

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

Build from source instead:

```sh
nix build github:jruz/agent-tool-check#source
```

See [AGENTS.md](AGENTS.md) for hook commands and configuration file locations.

## License

Licensed under the [European Union Public Licence v1.2](LICENSE).
