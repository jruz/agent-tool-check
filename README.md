# agent-hooks

Hooks for Claude Code, Codex, and Pi.

## Install with Nix

Requires Nix with flakes enabled.

```sh
nix profile add github:jruz/agent-hooks
```

Installs a published binary without compiling Rust. Supports Linux and macOS on x86_64 and aarch64.

Verify the installation:

```sh
agent-hooks --version
```

Upgrade:

```sh
nix profile upgrade agent-hooks
```

Run without installing:

```sh
nix run github:jruz/agent-hooks -- --help
```

Build from source instead:

```sh
nix build github:jruz/agent-hooks#source
```

See [AGENTS.md](AGENTS.md) for hook commands and configuration file locations.

## License

Licensed under the [European Union Public Licence v1.2](LICENSE).
