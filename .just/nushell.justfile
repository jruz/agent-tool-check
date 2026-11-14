set working-directory := '..'

test *ARGS:
    nu scripts/nushell-test.nu . {{ ARGS }}

check:
    nix develop ./.tools --command \
        just --justfile .tools/justfile nushell-check

fix:
    nix develop ./.tools --command \
        just --justfile .tools/justfile nushell-fix
