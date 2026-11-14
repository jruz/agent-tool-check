use std/assert

const CHECKER = path self | path dirname | path join architecture-check.nu

def fixture []: nothing -> path {
    let root = mktemp --directory
    mkdir ($root | path join src commands) ($root | path join tests commands)
    for name in [
        src/main.rs
        src/commands/mod.rs
        src/commands/claude.rs
        tests/main.rs
        tests/commands/mod.rs
        tests/commands/claude.rs
    ] {
        '' | save ($root | path join $name)
    }
    "enum Command {\n    Claude,\n}\n" | save --force ($root | path join src main.rs)
    $root
}

def invoke [root: path]: nothing -> record {
    ^nu $CHECKER ($root | path join src) | complete
}

def test-accepts-template-layout-and-fixtures []: nothing -> nothing {
    let root = fixture
    for name in [src/core src/libs src/utils tests/utils tests/fixtures/nested] {
        mkdir ($root | path join $name)
    }
    '{}' | save ($root | path join tests fixtures nested input.json)
    '' | save ($root | path join tests utils mod.rs)
    let result = invoke $root
    assert equal $result.exit_code 0 $result.stderr
    rm --permanent --recursive $root
}

def test-rejects-unexpected-source-roots []: nothing -> nothing {
    for name in [shared pre_tool_use .hidden] {
        let root = fixture
        mkdir ($root | path join src $name)
        let result = invoke $root
        assert equal $result.exit_code 1
        assert str contains $result.stderr $"($name): unexpected entry"
        rm --permanent --recursive $root
    }
}

def test-rejects-unexpected-source-files []: nothing -> nothing {
    let root = fixture
    '' | save ($root | path join src pre_tool_use.rs)
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'pre_tool_use.rs: unexpected entry'
    rm --permanent --recursive $root
}

def test-requires-entrypoints-and-command-modules []: nothing -> nothing {
    for name in [src/main.rs src/commands/mod.rs tests/main.rs tests/commands/mod.rs] {
        let root = fixture
        rm --permanent ($root | path join $name)
        let result = invoke $root
        assert equal $result.exit_code 1
        assert str contains $result.stderr $"($name): require file"
        rm --permanent --recursive $root
    }
}

def test-requires-test-for-every-command []: nothing -> nothing {
    let root = fixture
    "enum Command {\n    Claude,\n    Version,\n}\n" | save --force ($root | path join src main.rs)
    '' | save ($root | path join src commands version.rs)
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'tests/commands/version.rs: require file'
    '' | save ($root | path join tests commands version.rs)
    let updated = invoke $root
    assert equal $updated.exit_code 0 $updated.stderr
    rm --permanent --recursive $root
}

def test-rejects-extra-command-tests []: nothing -> nothing {
    let root = fixture
    '' | save ($root | path join tests commands missing.rs)
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'missing.rs: unexpected entry'
    rm --permanent --recursive $root
}

def test-rejects-unexpected-test-roots []: nothing -> nothing {
    for name in [support policies .hidden] {
        let root = fixture
        mkdir ($root | path join tests $name)
        let result = invoke $root
        assert equal $result.exit_code 1
        assert str contains $result.stderr $"($name): unexpected entry"
        rm --permanent --recursive $root
    }
}

def test-rejects-root-level-test-files []: nothing -> nothing {
    let root = fixture
    '' | save ($root | path join tests cli_test.rs)
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'cli_test.rs: unexpected entry'
    rm --permanent --recursive $root
}

def test-rejects-wrong-entry-kinds []: nothing -> nothing {
    let root = fixture
    '' | save ($root | path join src utils)
    mkdir ($root | path join tests fixtures)
    mkdir ($root | path join tests utils)
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'src/utils: expected dir'
    rm --permanent --recursive $root
}

def test-preserves-layer-dependency-checks []: nothing -> nothing {
    let root = fixture
    mkdir ($root | path join src utils)
    'use crate::commands::claude;' | save ($root | path join src utils wrong.rs)
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'utils cannot call commands::claude'
    rm --permanent --recursive $root
}

def test-preserves-entrypoint-dependency-checks []: nothing -> nothing {
    let root = fixture
    'use crate::core::hook;' | save --force ($root | path join src main.rs)
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'main cannot call core'
    rm --permanent --recursive $root
}

def test-requires-source-file-for-cli-variant []: nothing -> nothing {
    let root = fixture
    "enum Command {\n    Claude,\n    Version,\n}\n" | save --force ($root | path join src main.rs)
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'src/commands/version.rs: require file'
    assert str contains $result.stderr 'tests/commands/version.rs: require file'
    rm --permanent --recursive $root
}

def test-maps-multiword-command-variants-without-trailing-commas []: nothing -> nothing {
    let root = fixture
    "enum Command {\n    Claude,\n    ListItems\n}\n" | save --force ($root | path join src main.rs)
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'src/commands/list_items.rs: require file'
    assert str contains $result.stderr 'tests/commands/list_items.rs: require file'
    for name in [src tests] {
        '' | save ($root | path join $name commands list_items.rs)
    }
    let updated = invoke $root
    assert equal $updated.exit_code 0 $updated.stderr
    rm --permanent --recursive $root
}

def test-rejects-modules-without-cli-variants []: nothing -> nothing {
    let root = fixture
    '' | save ($root | path join src commands unused.rs)
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'unused.rs: unexpected entry'
    rm --permanent --recursive $root
}

def test-requires-command-inventory []: nothing -> nothing {
    let root = fixture
    '' | save --force ($root | path join src main.rs)
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'require a Command enum'
    rm --permanent --recursive $root
}

def unit-fixture []: string -> path {
    let code = $in
    let root = fixture
    mkdir ($root | path join src core)
    $code | save ($root | path join src core sample.rs)
    $root
}

def test-flags-direct-private-function-tests []: nothing -> nothing {
    for call in ['hidden()' 'super::hidden()' 'self::hidden()' 'hidden::<u8>()'] {
        let root = ('fn hidden<T>() {}
#[cfg(test)]
mod tests {
    #[test]
    fn checks_hidden() {
        CALL;
    }
}' | str replace CALL $call | unit-fixture)
        let result = invoke $root
        assert equal $result.exit_code 1
        assert str contains $result.stderr 'sample.rs:6: inline tests call private function hidden'
        rm --permanent --recursive $root
    }
}

def test-checks-test-modules-with-other-names []: nothing -> nothing {
    let root = 'fn hidden() {}
#[cfg(test)]
mod checks {
    #[test]
    fn checks_hidden() { super::hidden(); }
}' | unit-fixture
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'private function hidden'
    rm --permanent --recursive $root
}

def test-checks-namespaced-test-attributes []: nothing -> nothing {
    let root = 'async fn hidden() {}
#[tokio::test]
async fn checks_hidden() { hidden().await; }
' | unit-fixture
    let result = invoke $root
    assert equal $result.exit_code 1
    assert str contains $result.stderr 'private function hidden'
    rm --permanent --recursive $root
}

def test-allows-public-function-tests-and-test-helpers []: nothing -> nothing {
    let root = 'fn hidden() {}
pub fn public() { hidden(); }
pub(crate) fn crate_visible() {}
pub(super) fn parent_visible() {}
#[cfg(test)]
mod tests {
    fn helper() {}
    #[test]
    fn checks_public() {
        public();
        crate_visible();
        parent_visible();
        helper();
    }
}' | unit-fixture
    let result = invoke $root
    assert equal $result.exit_code 0 $result.stderr
    rm --permanent --recursive $root
}

def test-ignores-other-module-calls-and-line-comments []: nothing -> nothing {
    let root = 'fn hidden() {}
#[cfg(test)]
mod tests {
    #[test]
    fn checks_public() {
        external::hidden();
        // hidden();
    }
}' | unit-fixture
    let result = invoke $root
    assert equal $result.exit_code 0 $result.stderr
    rm --permanent --recursive $root
}

def test-flags-private-async-and-const-functions []: nothing -> nothing {
    for qualifier in [async const unsafe] {
        let root = ('QUALIFIER fn hidden() {}
#[cfg(test)]
mod tests {
    #[test]
    fn checks_hidden() { super::hidden(); }
}' | str replace QUALIFIER $qualifier | unit-fixture)
        let result = invoke $root
        assert equal $result.exit_code 1
        assert str contains $result.stderr 'private function hidden'
        rm --permanent --recursive $root
    }
}
