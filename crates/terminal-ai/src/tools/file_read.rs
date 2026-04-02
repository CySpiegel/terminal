use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "file_read".to_string(),
        description: "Read the contents of a file at the given path. Returns the file content with line numbers.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute path to the file to read"
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (0-based)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read"
                }
            },
            "required": ["path"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'path'".to_string()))?;

    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| ToolError::Execution(format!("failed to read {}: {}", path, e)))?;

    let offset = args["offset"].as_u64().unwrap_or(0) as usize;
    let limit = args["limit"].as_u64().unwrap_or(2000) as usize;

    let lines: Vec<String> = content
        .lines()
        .skip(offset)
        .take(limit)
        .enumerate()
        .map(|(i, line)| format!("{}\t{}", offset + i + 1, line))
        .collect();

    Ok(lines.join("\n"))
}
