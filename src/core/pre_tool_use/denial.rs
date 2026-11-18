use serde::Serialize;
use serde_json::Value;

use crate::utils::redact;

pub struct Block {
    pub context: Result<Context, String>,
    pub reason: String,
    pub text: String,
}

pub struct Context {
    pub call_id: Option<String>,
    pub input: Value,
    pub project: Option<String>,
    pub session: Option<String>,
    pub tool: String,
}

pub struct Pending {
    pub agent: &'static str,
    pub context: Context,
    pub reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub agent: &'static str,
    pub call_id: Option<String>,
    pub command: Option<String>,
    pub executable: Option<String>,
    pub project: String,
    pub reason: String,
    pub session: Option<String>,
    pub timestamp: String,
    pub tool: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Directory(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("project directory is not UTF-8: {0:?}")]
    ProjectEncoding(std::ffi::OsString),
    #[error(transparent)]
    Redact(#[from] redact::Error),
}

pub fn entry(pending: Pending) -> Result<Value, Error> {
    let Pending {
        agent,
        context,
        reason,
    } = pending;
    let command = invocation(&context.tool, &context.input)?;
    let executable = context
        .input
        .get("command")
        .and_then(Value::as_str)
        .and_then(|text| {
            let first = text.split_whitespace().next()?;
            let valid = first.starts_with(|character: char| character.is_ascii_alphabetic())
                && first.chars().all(|character| {
                    character.is_ascii_alphanumeric() || "_./-".contains(character)
                });
            valid.then(|| first.to_owned())
        });
    let project = match context.project {
        Some(project) => project,
        None => std::env::current_dir()?
            .into_os_string()
            .into_string()
            .map_err(Error::ProjectEncoding)?,
    };
    let entry = Entry {
        agent,
        timestamp: chrono::Utc::now().to_rfc3339(),
        project,
        session: context.session,
        call_id: context.call_id,
        tool: context.tool,
        executable,
        command,
        reason: redact::command(&reason)?,
    };
    let value = serde_json::to_value(entry)?;
    Ok(value)
}

fn invocation(tool: &str, input: &Value) -> Result<Option<String>, Error> {
    if let Some(command) = input.get("command").and_then(Value::as_str) {
        let command = redact::command(command)?;
        let command = Some(command);
        return Ok(command);
    }
    let path = input
        .get("path")
        .or_else(|| input.get("file_path"))
        .and_then(Value::as_str);
    match path {
        Some(path) => {
            let path = serde_json::to_string(path)?;
            let command = Some(format!("{tool} {path}"));
            Ok(command)
        }
        None => Ok(None),
    }
}
