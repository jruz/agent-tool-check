use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

use super::{denial, filters};
use crate::libs::response;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse Pi PreToolUse JSON: {0}")]
    Json(serde_json::Error),
    #[error("failed to serialize Pi PreToolUse response: {0}")]
    Serialize(serde_json::Error),
}

#[derive(Deserialize)]
struct ToolCall {
    input: Value,
    #[serde(rename = "toolName")]
    tool_name: String,
}

#[derive(Deserialize)]
struct Metadata {
    #[serde(rename = "toolCallId")]
    call_id: Option<String>,
    cwd: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

#[derive(Deserialize)]
struct BashInput {
    command: String,
}

#[derive(Deserialize)]
struct ReadInput {
    path: String,
}

#[derive(Deserialize)]
struct GrepInput {
    path: Option<String>,
}

pub fn run(stdin: &str) -> Result<Option<denial::Block>, Error> {
    let tool_call = parse_input(stdin)?;
    let result = match tool_call.tool_name.as_str() {
        "bash" => run_bash(&tool_call.input),
        "read" => run_read(&tool_call.input),
        "grep" => run_grep(&tool_call.input),
        _ => Ok(None),
    }?;
    let block = result.map(|(text, reason)| {
        let context = serde_json::from_str::<Metadata>(stdin)
            .map(|metadata| denial::Context {
                project: metadata.cwd,
                session: metadata.session_id,
                call_id: metadata.call_id,
                tool: tool_call.tool_name,
                input: tool_call.input,
            })
            .map_err(|err| err.to_string());
        denial::Block {
            context,
            reason,
            text,
        }
    });
    Ok(block)
}

fn parse_input(stdin: &str) -> Result<ToolCall, Error> {
    serde_json::from_str(stdin).map_err(Error::Json)
}

fn run_bash(input: &Value) -> Result<Option<(String, String)>, Error> {
    let bash = BashInput::deserialize(input).map_err(Error::Json)?;
    let result = filters::bash::run(&bash.command);
    build_response(result)
}

fn run_read(input: &Value) -> Result<Option<(String, String)>, Error> {
    let read = ReadInput::deserialize(input).map_err(Error::Json)?;
    let result = filters::secrets::run(&read.path);
    build_response(result)
}

fn run_grep(input: &Value) -> Result<Option<(String, String)>, Error> {
    let grep = GrepInput::deserialize(input).map_err(Error::Json)?;
    match grep.path {
        Some(path) => {
            let result = filters::secrets::run(&path);
            build_response(result)
        }
        None => Ok(None),
    }
}

fn build_response(result: Result<(), String>) -> Result<Option<(String, String)>, Error> {
    let Err(reason) = result else { return Ok(None) };
    let response_reason = reason.clone();
    let text = response::pre_tool_use_pi_block(response_reason).map_err(Error::Serialize)?;
    let block = Some((text, reason));
    Ok(block)
}
