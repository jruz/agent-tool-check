import '.just/release.justfile'

mod bun '.just/bun.justfile'
mod rust '.just/rust.justfile'
mod ts '.just/ts.justfile'

skill-pull:
    bun "$HOME/.pi/agent/skills/typescript/skill-pull/main.ts"

setup:
    just ts::install

install:
    just rust::install

check:
    just nu-check
    just nix-check
    just woodpecker-check
    just rust::check
    just ts::install
    just ts::check
    just bun::check

test:
    just rust::test
    just bun::test
    just ts::oxlint-rules-check

run *ARGS='':
    just rust::run {{ARGS}}

logs:
    nvim ~/.local/state/agent-hooks/hooks.log

ts-dependencies-add +PACKAGES:
    sandbox pnpm add --save-dev --save-exact {{PACKAGES}}

bun-install:
    just ts::install

bun-test:
    just bun::test

bun-check:
    just ts::check

watch:
    just rust::watch

fix:
    just nix-fix
    just rust::fix
    just ts::fix

nu-check:
    #!/usr/bin/env nu
    for script in (glob scripts/**/*.nu) {
        if not (nu-check --debug $script) {
            error make {msg: $"invalid script: ($script)"}
        }
    }
    ^nu-lint --config nu-lint.toml scripts

# ──────────────────────────────── nix ────────────────────────────────

nix-check:
    fd \
        --extension nix \
        --exec nixfmt --check
    statix check .
    deadnix --fail .

nix-fix:
    fd \
        --extension nix \
        --exec nixfmt
    statix fix .
    deadnix --edit .

# ────────────────────────────── woodpecker ──────────────────────────────

woodpecker-check:
    woodpecker-cli lint \
        --strict \
        .woodpecker/

