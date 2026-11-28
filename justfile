mod git '.just/git.just'

mod logs '.just/logs.justfile'

mod just '.just/just.just'

mod tools '.just/tools.just'

mod bun '.just/bun.justfile'

mod nushell '.just/nushell.justfile'

mod rust '.just/rust.just'

mod ts '.just/ts.justfile'

skill-pull:
    bun "$HOME/.pi/agent/skills/typescript/skill-pull/main.ts"

setup:
    direnv allow
    just git::hooks-setup
    just ts::install

install:
    just rust::install

check:
    just just::fmt-check
    just oxfmt::fmt-check
    just rust::check
    just ts::install
    just ts::check
    just bun::check

test:
    just nushell::test
    just rust::test
    just bun::test
    just ts::oxlint-rules-check

run *ARGS='':
    just rust::run {{ ARGS }}

[positional-arguments]
denials-list *ARGS:
    sandbox bun scripts/denials-list.ts "$@"

ts-dependencies-add +PACKAGES:
    sandbox pnpm add --save-dev --save-exact {{ PACKAGES }}

bun-install:
    just ts::install

bun-test:
    just bun::test

bun-check:
    just ts::check

watch:
    just rust::watch

fix:
    just just::fmt-fix
    just oxfmt::fmt-fix
    just rust::fix
    just ts::fix

mod oxfmt '.just/oxfmt.just'

[positional-arguments]
release +ARGS:
    nu scripts/release.nu "$@"
