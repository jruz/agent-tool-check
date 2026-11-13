const credential =
  /\b([\w-]*(?:token|password|passwd|secret|api[_-]?key|authorization)[\w-]*\s*=\s*)(?:"[^"\n]*"|'[^'\n]*'|[^\s;|&]+)/giu;
const flag =
  /((?:--[\w-]*(?:token|password|passwd|secret|api[-_]?key)|--header|--user|-H|-u)\b(?:=|\s+))(?:"[^"\n]*"|'[^'\n]*'|[^\s;|&]+)/giu;
const basicAuth = /(https?:\/\/)[^\s/@]+:[^\s/@]+@/giu;

export const redactCommand = (command: string): string =>
  command
    .replace(credential, "$1[REDACTED]")
    .replace(flag, "$1[REDACTED]")
    .replace(basicAuth, "$1[REDACTED]@");
