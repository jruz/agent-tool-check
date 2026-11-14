use lib/log.nu

const LAYERS = {
    commands: {calls: [core utils], siblings: false}
    core: {calls: [libs utils], siblings: true}
    libs: {calls: [utils], siblings: false}
    utils: {calls: [], siblings: false}
}

const ENTRYPOINT_CALLS = [commands utils]

const SOURCE_ENTRIES = {
    'main.rs': file
    commands: dir
    core: dir
    libs: dir
    utils: dir
}

const TEST_ENTRIES = {
    'main.rs': file
    commands: dir
    utils: dir
    fixtures: dir
}

def required-violations [root: path, entries: record]: nothing -> list<string> {
    ($entries
        | items {|name, kind|
            let path = $root | path join $name
            if ($path | path type) != $kind {
                $"($path): require ($kind)"
            }
        }
        | compact)
}

def root-violations [root: path, entries: record]: nothing -> list<string> {
    if ($root | path type) != dir {
        return [$"($root): require directory"]
    }
    (ls --all $root
        | each {|entry|
            let name = $entry.name | path basename
            if $entries not-has $name {
                return $"($entry.name): unexpected entry; allowed: ($entries | columns | str join ', ')"
            }
            let expected = $entries | transpose name kind | where name == $name | get kind | first
            if $entry.type != $expected {
                $"($entry.name): expected ($expected)"
            }
        }
        | compact)
}

def command-names [source: path]: nothing -> list<string> {
    let entrypoint = $source | path join main.rs
    if ($entrypoint | path type) != file {
        return []
    }
    (open --raw $entrypoint
        | lines
        | skip until {|line| $line =~ '^enum Command\s*\{' }
        | skip 1
        | take until {|line| $line =~ '^\}' }
        | parse --regex '^\s+(?<name>[A-Z]\w*)\s*,?\s*$'
        | get name
        | str snake-case
        | sort)
}

def command-violations [root: path, commands: list<string>]: nothing -> list<string> {
    if ($root | path type) != dir {
        return [$"($root): require command directory"]
    }
    let files = $commands | each {|name| $"($name).rs" } | prepend mod.rs
    let entries = $files | each {|name| {name: $name, kind: file} } | transpose --header-row --as-record
    (root-violations $root $entries
        | append (required-violations $root $entries))
}

def layout-violations [source: path, tests: path]: nothing -> list<string> {
    let commands = command-names $source
    [
        ...(root-violations $source $SOURCE_ENTRIES)
        ...(required-violations $source {'main.rs': file, commands: dir})
        ...(command-violations ($source | path join commands) $commands)
        ...(root-violations $tests $TEST_ENTRIES)
        ...(required-violations $tests {'main.rs': file, commands: dir})
        ...(command-violations ($tests | path join commands) $commands)
        ...(if ($commands | is-empty) { [$"($source)/main.rs: require a Command enum with one unit variant per command"] } else { [] })
    ]
}

def file-calls [pattern: string]: path -> table {
    let file = ($in | path expand)
    (open $file
        | lines
        | enumerate
        | each {|row|
            $row.item
                | parse --regex $pattern
                | insert file $file
                | insert line ($row.index + 1)
                | insert text $row.item
        }
        | flatten)
}

def owner [layer_root: path]: path -> string {
    path relative-to $layer_root | path split | first | str replace --regex '\.rs$' ''
}

def is-allowed [layer: record, module: string]: record -> bool {
    let call = $in
    if $LAYERS not-has $call.target {
        return true
    }
    if $call.target == $layer.name {
        return ($layer.rule.siblings or $call.module == $module)
    }
    $call.target in $layer.rule.calls
}

def layer-violations []: record -> list<string> {
    let layer = $in
    (glob ($layer.root | path join '**/*.rs')
        | each {|file|
            let module = ($file | owner $layer.root)
            ($file
                | file-calls 'crate::(?<target>\w+)::(?<module>\w+)'
                | where not ($it | is-allowed $layer $module))
        }
        | flatten
        | each {|call| $"($call.file):($call.line): ($layer.name) cannot call ($call.target)::($call.module)" })
}

def entrypoint-violations [entrypoint: path]: nothing -> list<string> {
    if not ($entrypoint | path exists) {
        return []
    }
    ($entrypoint
        | file-calls '\b(?<target>commands|core|libs|utils)::'
        | where target not-in $ENTRYPOINT_CALLS
        | each {|call| $"($call.file):($call.line): main cannot call ($call.target)" })
}

def private-test-violations [file: path]: nothing -> list<string> {
    let tests = $file | file-calls '^\s*#\[(?<attribute>cfg\(test\)|(?:\w+::)*test(?:\([^]]*\))?)\]'
    if ($tests | is-empty) {
        return []
    }
    let boundary = $tests | get line | first
    let functions = ($file
        | file-calls '^\s*(?:(?:async|const|unsafe)\s+)*fn\s+(?<name>\w+)\s*(?:<|\()'
        | where line < $boundary
        | get name)
    if ($functions | is-empty) {
        return []
    }
    let names = $functions | str join '|'
    let pattern = ['(?:^|[^\w:.])(?:super::|self::)?(?<name>' $names ')\s*(?:::<[^>\n]*>)?\s*\('] | str join
    ($file
        | file-calls $pattern
        | where line > $boundary
        | where not ($it.text | str trim | str starts-with '//')
        | each {|call| $"($call.file):($call.line): inline tests call private function ($call.name); test its public API" })
}

# Check source layout, command test files, layer dependencies, and direct private-function tests.
def main [
    source: path # source directory containing the layer folders
]: nothing -> nothing {
    let tests = $source | path expand | path dirname | path join tests
    let violations = ($LAYERS
        | items {|name, rule| {root: ($source | path join $name | path expand), name: $name, rule: $rule} }
        | where ($it.root | path exists)
        | each { layer-violations }
        | flatten
        | prepend (entrypoint-violations ($source | path join main.rs))
        | prepend (layout-violations $source $tests)
        | append (glob ($source | path join '**/*.rs') | each { private-test-violations $in } | flatten))
    if ($violations | is-not-empty) {
        for violation in $violations {
            log error $violation
        }
        exit 1
    }
    log ok "source layout, command test files, and layer dependencies conform"
}
