use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "agent".to_string(),
        description: "Spawn a sub-agent to handle a complex, multi-step task autonomously. \
            The sub-agent gets its own conversation and tool access. Use this for tasks that \
            require multiple tool calls, deep research, or parallel work. Available agent \
            types: 'general' (default), 'code_reviewer', 'test_writer', 'refactorer', \
            'documenter', 'debugger', 'searcher'."
            .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "A detailed description of the task for the sub-agent. Include all necessary context — the agent starts fresh with no prior conversation."
                },
                "role": {
                    "type": "string",
                    "description": "The agent type/role. One of: 'general', 'code_reviewer', 'test_writer', 'refactorer', 'documenter', 'debugger', 'searcher'. Default: 'general'."
                }
            },
            "required": ["task"]
        }),
    }
}

/// Note: The agent tool cannot be executed as a simple async function because
/// it needs access to the AiConfig to create a sub-agent. Instead, the agent
/// loop in agent.rs handles this tool specially by intercepting "agent" tool
/// calls and routing them through the AgentCoordinator.
///
/// This execute function is a placeholder that returns an error directing
/// the caller to use the AgentCoordinator.
pub async fn execute(args: serde_json::Value) -> ToolResult {
    let task = args["task"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'task'".to_string()))?;

    let role = args["role"].as_str().unwrap_or("general");

    // In practice, the agent loop intercepts this tool call and routes it
    // through AgentCoordinator. This fallback just returns the task description.
    Err(ToolError::Execution(format!(
        "Agent tool must be handled by the agent loop. Role: {}, Task: {}",
        role, task
    )))
}
