use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "think".to_string(),
        description: "Use this tool to think through complex problems step-by-step before taking action. Your thoughts are returned but not shown to the user — use this as a scratchpad for reasoning, planning, and analysis.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "thought": {
                    "type": "string",
                    "description": "Your internal reasoning and analysis"
                }
            },
            "required": ["thought"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let thought = args["thought"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'thought'".to_string()))?;

    Ok(thought.to_string())
}
