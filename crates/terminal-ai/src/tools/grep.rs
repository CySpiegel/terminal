use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;
use tokio::process::Command;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "grep".to_string(),
        description: "Search file contents using a regex pattern. Returns matching lines with file paths and line numbers.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "File or directory to search in (default: current directory)"
                },
                "file_type": {
                    "type": "string",
                    "description": "File type filter (e.g., 'rs', 'py', 'js')"
                },
                "case_insensitive": {
                    "type": "boolean",
                    "description": "Case-insensitive search (default: false)"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 50)"
                }
            },
            "required": ["pattern"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let pattern = args["pattern"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'pattern'".to_string()))?;

    let path = args["path"].as_str().unwrap_or(".");
    let max_results = args["max_results"].as_u64().unwrap_or(50);
    let case_insensitive = args["case_insensitive"].as_bool().unwrap_or(false);

    let mut cmd = Command::new("grep");
    cmd.arg("-rn").arg("--color=never");

    if case_insensitive {
        cmd.arg("-i");
    }

    if let Some(file_type) = args["file_type"].as_str() {
        cmd.arg("--include").arg(format!("*.{}", file_type));
    }

    cmd.arg("-E").arg(pattern).arg(path);

    let output = cmd
        .output()
        .await
        .map_err(|e| ToolError::Execution(format!("grep failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().take(max_results as usize).collect();

    if lines.is_empty() {
        Ok("No matches found.".to_string())
    } else {
        Ok(lines.join("\n"))
    }
}
