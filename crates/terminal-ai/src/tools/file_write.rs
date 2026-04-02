use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "file_write".to_string(),
        description: "Write content to a file, creating it if it doesn't exist or overwriting if it does.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["path", "content"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'path'".to_string()))?;
    let content = args["content"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'content'".to_string()))?;

    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(path).parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            ToolError::Execution(format!("failed to create directory {}: {}", parent.display(), e))
        })?;
    }

    tokio::fs::write(path, content)
        .await
        .map_err(|e| ToolError::Execution(format!("failed to write {}: {}", path, e)))?;

    Ok(format!("Successfully wrote {} bytes to {}", content.len(), path))
}
