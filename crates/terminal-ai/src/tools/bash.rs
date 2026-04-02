use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;
use tokio::process::Command;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "bash".to_string(),
        description: "Execute a shell command and return its output (stdout + stderr).".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "Timeout in milliseconds (default: 120000)"
                }
            },
            "required": ["command"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let command = args["command"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'command'".to_string()))?;

    let timeout_ms = args["timeout_ms"].as_u64().unwrap_or(120_000);

    let output = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        Command::new("sh").arg("-c").arg(command).output(),
    )
    .await
    .map_err(|_| ToolError::Execution(format!("command timed out after {}ms", timeout_ms)))?
    .map_err(|e| ToolError::Execution(format!("failed to execute command: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = String::new();
    if !stdout.is_empty() {
        result.push_str(&stdout);
    }
    if !stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str("STDERR:\n");
        result.push_str(&stderr);
    }

    if !output.status.success() {
        result.push_str(&format!(
            "\nExit code: {}",
            output.status.code().unwrap_or(-1)
        ));
    }

    // Truncate very long output
    if result.len() > 100_000 {
        result.truncate(100_000);
        result.push_str("\n... (output truncated)");
    }

    Ok(result)
}
