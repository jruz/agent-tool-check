const sensitiveReasons = new Set([
  "Never access .env files directly",
  "Access to sensitive file blocked",
  "Access to system secret file blocked",
]);
const bashReasons = new Set([
  ...sensitiveReasons,
  "sed option requires a value",
  "Never include Co-Authored-By in commit messages",
  "Invalid commit title, follow the Skill(commit) format",
  "Could not extract commit title",
  "Never print environment variables",
  "Shell option requires a value",
  "Variable output permits plain lowercase names, HOME, USER, PATH, and exit status ($?).",
  "Never print variables with TOKEN, KEY, SECRET, PASSWORD, or PASSPHRASE name parts (case-insensitive). Unsupported output expansions remain blocked.",
  "Must be prefixed by `sandbox` (e.g. `sandbox cargo build`). Use `just` recipes when available.",
  "Use bare `sandbox`, not an absolute or relative wrapper path.",
  "Dynamic executable names must be invoked through `sandbox`",
  "find execution clauses must end with `;` or `+`",
  "Could not inspect this command because parsing failed.",
  "Use `just install` instead",
  "Use `just <task>` instead of `npm run`",
  "Use `just playwright-install` instead",
  "Use `just <task>` instead of `npx run`",
  "Access to sensitive command blocked",
  "Direct SOPS decryption is blocked. Read the encrypted YAML directly to inspect keys and unencrypted values. Use `just sops-decrypt-dev` to generate `.env` and `just dotenv-show` for masked inspection, where these recipes exist.",
  "Use just run instead",
  "Access to macOS security command blocked",
  "Access to shell history blocked",
  "Never use --all with git filter-branch — it overwrites backup branches. Target specific branches instead.",
  "Never read container environment variables",
  "Don't skip hooks",
  "Only run update-snapshots after investigating the test results and confirming the diff is not a bug. Ask the user to run this command manually.",
]);

export const hookError = (tool: string, reason: string): string | null => {
  if (["bash", "Bash", "exec", "exec_command", "shell_command"].includes(tool))
    return bashReasons.has(reason) ? reason : null;
  if (["read", "Read", "grep", "Grep"].includes(tool))
    return sensitiveReasons.has(reason) ? reason : null;
  return null;
};

export const knownHookError = (reason: string): boolean =>
  bashReasons.has(reason);
