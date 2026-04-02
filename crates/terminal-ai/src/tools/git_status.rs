use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;
use tokio::process::Command;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "git_status".to_string(),
        description: "Run git commands: status, diff, log, or branch. Returns the command output.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "subcommand": {
                    "type": "string",
                    "description": "Git subcommand to run",
                    "enum": ["status", "diff", "log", "branch"]
                },
                "args": {
                    "type": "string",
                    "description": "Additional arguments to pass to the git subcommand"
                }
            },
            "required": ["subcommand"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let subcommand = args["subcommand"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'subcommand'".to_string()))?;

    // Validate the subcommand
    match subcommand {
        "status" | "diff" | "log" | "branch" => {}
        other => {
            return Err(ToolError::InvalidArgs(format!(
                "invalid subcommand '{}': must be one of: status, diff, log, branch",
                other
            )));
        }
    }

    let mut cmd = Command::new("git");
    cmd.arg(subcommand);

    // Add extra args if provided
    if let Some(extra_args) = args["args"].as_str() {
        for arg in extra_args.split_whitespace() {
            cmd.arg(arg);
        }
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| ToolError::Execution(format!("failed to run git: {}", e)))?;

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
        result.push_str(&stderr);
    }

    if result.is_empty() {
        result = "No output.".to_string();
    }

    // Truncate very long output (e.g., large diffs)
    if result.len() > 100_000 {
        result.truncate(100_000);
        result.push_str("\n... (output truncated)");
    }

    Ok(result)
}
