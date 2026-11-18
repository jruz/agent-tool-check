use super::denial;

#[derive(serde::Deserialize)]
pub struct Hook {
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub tool_input: serde_json::Value,
    pub tool_name: String,
    pub tool_use_id: Option<String>,
}

pub fn from_hook(hook: Hook) -> denial::Context {
    denial::Context {
        project: hook.cwd,
        session: hook.session_id,
        call_id: hook.tool_use_id,
        tool: hook.tool_name,
        input: hook.tool_input,
    }
}
