use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;
use tokio::process::Command;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "archive".to_string(),
        description: "Create or extract archive files. Supports tar.gz and zip formats.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Action to perform",
                    "enum": ["create", "extract"]
                },
                "path": {
                    "type": "string",
                    "description": "Path to the archive file (for extract) or the source file/directory (for create)"
                },
                "destination": {
                    "type": "string",
                    "description": "Destination path — output archive file (for create) or extraction directory (for extract). Defaults to current directory for extract."
                },
                "format": {
                    "type": "string",
                    "description": "Archive format (default: inferred from file extension)",
                    "enum": ["tar.gz", "zip"]
                }
            },
            "required": ["action", "path"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let action = args["action"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'action'".to_string()))?;

    let path = args["path"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'path'".to_string()))?;

    match action {
        "create" => create_archive(path, &args).await,
        "extract" => extract_archive(path, &args).await,
        other => Err(ToolError::InvalidArgs(format!(
            "invalid action '{}': must be 'create' or 'extract'",
            other
        ))),
    }
}

fn detect_format(path: &str, args: &serde_json::Value) -> String {
    if let Some(fmt) = args["format"].as_str() {
        return fmt.to_string();
    }
    let lower = path.to_lowercase();
    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        "tar.gz".to_string()
    } else if lower.ends_with(".zip") {
        "zip".to_string()
    } else {
        "tar.gz".to_string()
    }
}

async fn create_archive(source_path: &str, args: &serde_json::Value) -> ToolResult {
    let destination = args["destination"].as_str();
    let format = detect_format(
        destination.unwrap_or(source_path),
        args,
    );

    let output_path = match destination {
        Some(dest) => dest.to_string(),
        None => {
            if format == "zip" {
                format!("{}.zip", source_path)
            } else {
                format!("{}.tar.gz", source_path)
            }
        }
    };

    let output = match format.as_str() {
        "tar.gz" => {
            Command::new("tar")
                .arg("-czf")
                .arg(&output_path)
                .arg(source_path)
                .output()
                .await
                .map_err(|e| ToolError::Execution(format!("failed to run tar: {}", e)))?
        }
        "zip" => {
            Command::new("zip")
                .arg("-r")
                .arg(&output_path)
                .arg(source_path)
                .output()
                .await
                .map_err(|e| ToolError::Execution(format!("failed to run zip: {}", e)))?
        }
        _ => {
            return Err(ToolError::InvalidArgs(format!(
                "unsupported format: {}",
                format
            )));
        }
    };

    if output.status.success() {
        Ok(format!("Created archive: {}", output_path))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ToolError::Execution(format!(
            "archive creation failed: {}",
            stderr.trim()
        )))
    }
}

async fn extract_archive(archive_path: &str, args: &serde_json::Value) -> ToolResult {
    let destination = args["destination"].as_str().unwrap_or(".");
    let format = detect_format(archive_path, args);

    let output = match format.as_str() {
        "tar.gz" => {
            Command::new("tar")
                .arg("-xzf")
                .arg(archive_path)
                .arg("-C")
                .arg(destination)
                .output()
                .await
                .map_err(|e| ToolError::Execution(format!("failed to run tar: {}", e)))?
        }
        "zip" => {
            Command::new("unzip")
                .arg("-o")
                .arg(archive_path)
                .arg("-d")
                .arg(destination)
                .output()
                .await
                .map_err(|e| ToolError::Execution(format!("failed to run unzip: {}", e)))?
        }
        _ => {
            return Err(ToolError::InvalidArgs(format!(
                "unsupported format: {}",
                format
            )));
        }
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(format!(
            "Extracted {} to {}\n{}",
            archive_path,
            destination,
            stdout.trim()
        ))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ToolError::Execution(format!(
            "extraction failed: {}",
            stderr.trim()
        )))
    }
}
