use crate::core::{agent, pre_tool_use};
use crate::utils::{failure, output};

pub fn run() -> Result<output::Output, failure::Error> {
    pre_tool_use::run(agent::Agent::Codex)
}
