use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;
use tokio::process::Command;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "process".to_string(),
        description: "List running processes or kill a process by PID. Can filter the process list by name pattern.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Action to perform",
                    "enum": ["list", "kill"]
                },
                "pid": {
                    "type": "integer",
                    "description": "Process ID to kill (required when action is 'kill')"
                },
                "pattern": {
                    "type": "string",
                    "description": "Filter processes by name pattern (used with 'list')"
                }
            },
            "required": ["action"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let action = args["action"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'action'".to_string()))?;

    match action {
        "list" => list_processes(&args).await,
        "kill" => kill_process(&args).await,
        other => Err(ToolError::InvalidArgs(format!(
            "invalid action '{}': must be 'list' or 'kill'",
            other
        ))),
    }
}

async fn list_processes(args: &serde_json::Value) -> ToolResult {
    let pattern = args["pattern"].as_str();

    let mut cmd = Command::new("ps");
    cmd.arg("aux");

    let output = cmd
        .output()
        .await
        .map_err(|e| ToolError::Execution(format!("failed to run ps: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Some(pat) = pattern {
        let pat_lower = pat.to_lowercase();
        let lines: Vec<&str> = stdout
            .lines()
            .enumerate()
            .filter(|(i, line)| *i == 0 || line.to_lowercase().contains(&pat_lower))
            .map(|(_, line)| line)
            .take(100)
            .collect();

        if lines.len() <= 1 {
            Ok(format!("No processes matching '{}'", pat))
        } else {
            Ok(lines.join("\n"))
        }
    } else {
        // Return first 50 lines to avoid overwhelming output
        let lines: Vec<&str> = stdout.lines().take(50).collect();
        let mut result = lines.join("\n");
        if stdout.lines().count() > 50 {
            result.push_str("\n... (truncated, use 'pattern' to filter)");
        }
        Ok(result)
    }
}

async fn kill_process(args: &serde_json::Value) -> ToolResult {
    let pid = args["pid"]
        .as_u64()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'pid' for kill action".to_string()))?;

    let output = Command::new("kill")
        .arg(pid.to_string())
        .output()
        .await
        .map_err(|e| ToolError::Execution(format!("failed to run kill: {}", e)))?;

    if output.status.success() {
        Ok(format!("Successfully sent SIGTERM to process {}", pid))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ToolError::Execution(format!(
            "failed to kill process {}: {}",
            pid,
            stderr.trim()
        )))
    }
}
