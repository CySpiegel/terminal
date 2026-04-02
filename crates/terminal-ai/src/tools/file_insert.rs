use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "file_insert".to_string(),
        description: "Insert text at a specific line number in a file. The content is inserted before the specified line. Use line 0 to prepend to the file.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute path to the file"
                },
                "line": {
                    "type": "integer",
                    "description": "Line number to insert before (1-based, or 0 to prepend)"
                },
                "content": {
                    "type": "string",
                    "description": "The text content to insert"
                }
            },
            "required": ["path", "line", "content"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'path'".to_string()))?;

    let line = args["line"]
        .as_u64()
        .ok_or_else(|| ToolError::InvalidArgs("missing or invalid 'line'".to_string()))?
        as usize;

    let content = args["content"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'content'".to_string()))?;

    let file_content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| ToolError::Execution(format!("failed to read {}: {}", path, e)))?;

    let mut lines: Vec<&str> = file_content.lines().collect();
    let total_lines = lines.len();

    // Line is 1-based; 0 means prepend
    let insert_index = if line == 0 {
        0
    } else if line - 1 > total_lines {
        total_lines
    } else {
        line - 1
    };

    // Split the content to insert into lines
    let new_lines: Vec<&str> = content.lines().collect();

    // Insert the new lines
    for (i, new_line) in new_lines.iter().enumerate() {
        lines.insert(insert_index + i, new_line);
    }

    let new_content = lines.join("\n");
    // Preserve trailing newline if original had one
    let new_content = if file_content.ends_with('\n') && !new_content.ends_with('\n') {
        format!("{}\n", new_content)
    } else {
        new_content
    };

    tokio::fs::write(path, &new_content)
        .await
        .map_err(|e| ToolError::Execution(format!("failed to write {}: {}", path, e)))?;

    let inserted_count = new_lines.len();
    Ok(format!(
        "Inserted {} line(s) at line {} in {}",
        inserted_count,
        if line == 0 { 1 } else { line },
        path
    ))
}
