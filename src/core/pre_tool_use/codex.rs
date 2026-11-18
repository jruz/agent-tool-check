use serde::Deserialize;
use thiserror::Error;

use crate::libs::response;

use super::{context, denial, filters};

#[derive(Error, Debug)]
pub enum CodexError {
    #[error("failed to parse Codex PreToolUse JSON: {0}")]
    Json(serde_json::Error),
    #[error("failed to serialize Codex PreToolUse response: {0}")]
    Serialize(serde_json::Error),
}

#[derive(Deserialize)]
#[serde(tag = "tool_name", rename_all = "PascalCase")]
enum HookInput {
    Bash {
        tool_input: BashInput,
    },
    #[serde(other)]
    Unsupported,
}

#[derive(Deserialize)]
struct BashInput {
    command: String,
}

pub fn run(stdin: &str) -> Result<Option<denial::Block>, CodexError> {
    let input: HookInput = serde_json::from_str(stdin).map_err(CodexError::Json)?;
    match input {
        HookInput::Bash { tool_input: bash } => {
            let result = filters::bash::run(&bash.command);
            build_response(result, stdin)
        }
        HookInput::Unsupported => Ok(None),
    }
}

fn build_response(
    result: Result<(), String>,
    stdin: &str,
) -> Result<Option<denial::Block>, CodexError> {
    let Err(reason) = result else { return Ok(None) };
    let context = serde_json::from_str::<context::Hook>(stdin)
        .map(context::from_hook)
        .map_err(|err| err.to_string());
    let response_reason = reason.clone();
    let text =
        response::pre_tool_use_codex_block(response_reason).map_err(CodexError::Serialize)?;
    let block = denial::Block {
        context,
        reason,
        text,
    };
    let block = Some(block);
    Ok(block)
}
