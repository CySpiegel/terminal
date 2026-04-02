use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "file_edit".to_string(),
        description: "Perform an exact string replacement in a file. The old_string must match exactly one location in the file.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute path to the file to edit"
                },
                "old_string": {
                    "type": "string",
                    "description": "The exact string to find and replace"
                },
                "new_string": {
                    "type": "string",
                    "description": "The string to replace it with"
                }
            },
            "required": ["path", "old_string", "new_string"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'path'".to_string()))?;
    let old_string = args["old_string"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'old_string'".to_string()))?;
    let new_string = args["new_string"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'new_string'".to_string()))?;

    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| ToolError::Execution(format!("failed to read {}: {}", path, e)))?;

    let count = content.matches(old_string).count();
    if count == 0 {
        return Err(ToolError::Execution(format!(
            "old_string not found in {}",
            path
        )));
    }
    if count > 1 {
        return Err(ToolError::Execution(format!(
            "old_string found {} times in {} — must be unique. Provide more context.",
            count, path
        )));
    }

    let new_content = content.replacen(old_string, new_string, 1);
    tokio::fs::write(path, &new_content)
        .await
        .map_err(|e| ToolError::Execution(format!("failed to write {}: {}", path, e)))?;

    Ok(format!("Successfully edited {}", path))
}
