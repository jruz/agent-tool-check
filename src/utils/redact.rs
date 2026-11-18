#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Regex(#[from] regex::Error),
}

pub fn command(command: &str) -> Result<String, Error> {
    let credential = regex::Regex::new(
        r#"(?i)\b([\w-]*(?:token|password|passwd|secret|api[_-]?key|authorization)[\w-]*\s*=\s*)(?:"[^"\n]*"|'[^'\n]*'|[^\s;|&]+)"#,
    )?;
    let flag = regex::Regex::new(
        r#"(?i)((?:--[\w-]*(?:token|password|passwd|secret|api[-_]?key)|--header|--user|-H|-u)\b(?:=|\s+))(?:"[^"\n]*"|'[^'\n]*'|[^\s;|&]+)"#,
    )?;
    let basic_auth = regex::Regex::new(r"(?i)(https?://)[^\s/@]+:[^\s/@]+@")?;
    let redacted = credential.replace_all(command, "${1}[REDACTED]");
    let redacted = flag.replace_all(&redacted, "${1}[REDACTED]");
    let redacted = basic_auth
        .replace_all(&redacted, "${1}[REDACTED]@")
        .into_owned();
    Ok(redacted)
}
