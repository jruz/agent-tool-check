set working-directory := '..'

state-directory := env('XDG_STATE_HOME', env('HOME') / '.local/state')
log-directory := state-directory / 'agent-tool-check'

errors-read:
    jq . {{ quote(log-directory / 'errors.log') }}

errors-clear:
    truncate --size 0 {{ quote(log-directory / 'errors.log') }}

denials-read:
    jq . {{ quote(log-directory / 'denial.log') }}

denials-clear:
    truncate --size 0 {{ quote(log-directory / 'denial.log') }}
