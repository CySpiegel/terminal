use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;
use tokio::process::Command;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "glob_search".to_string(),
        description: "Find files matching a glob pattern. Returns matching file paths.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern (e.g., '**/*.rs', 'src/**/*.ts')"
                },
                "path": {
                    "type": "string",
                    "description": "Base directory to search in (default: current directory)"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 100)"
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

    let base_path = args["path"].as_str().unwrap_or(".");
    let max_results = args["max_results"].as_u64().unwrap_or(100);

    // Use find as a cross-platform fallback
    let output = Command::new("find")
        .arg(base_path)
        .arg("-name")
        .arg(pattern)
        .arg("-type")
        .arg("f")
        .output()
        .await
        .map_err(|e| ToolError::Execution(format!("find failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<&str> = stdout.lines().take(max_results as usize).collect();

    if files.is_empty() {
        Ok("No files found.".to_string())
    } else {
        Ok(files.join("\n"))
    }
}
