use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "list_directory".to_string(),
        description: "List directory contents with metadata including file type (file/dir/symlink) and size.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute path to the directory to list"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "List contents recursively (default: false)"
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum recursion depth when recursive is true (default: 3)"
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

    let recursive = args["recursive"].as_bool().unwrap_or(false);
    let max_depth = args["max_depth"].as_u64().unwrap_or(3) as usize;

    let path = std::path::PathBuf::from(path);
    if !path.exists() {
        return Err(ToolError::Execution(format!(
            "path does not exist: {}",
            path.display()
        )));
    }
    if !path.is_dir() {
        return Err(ToolError::Execution(format!(
            "not a directory: {}",
            path.display()
        )));
    }

    let mut entries = Vec::new();
    let max_entries = 1000;

    if recursive {
        list_recursive(&path, &path, 0, max_depth, &mut entries, max_entries).await?;
    } else {
        list_flat(&path, &mut entries, max_entries).await?;
    }

    if entries.is_empty() {
        Ok("Directory is empty.".to_string())
    } else {
        Ok(entries.join("\n"))
    }
}

async fn list_flat(
    dir: &std::path::Path,
    entries: &mut Vec<String>,
    max_entries: usize,
) -> Result<(), ToolError> {
    let mut read_dir = tokio::fs::read_dir(dir)
        .await
        .map_err(|e| ToolError::Execution(format!("failed to read directory: {}", e)))?;

    let mut items = Vec::new();
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| ToolError::Execution(format!("failed to read entry: {}", e)))?
    {
        items.push(entry);
    }

    // Sort by name
    items.sort_by_key(|e| e.file_name());

    for entry in items {
        if entries.len() >= max_entries {
            entries.push("... (truncated)".to_string());
            break;
        }

        let metadata = entry.metadata().await;
        let name = entry.file_name().to_string_lossy().to_string();
        let line = format_entry(&name, &metadata);
        entries.push(line);
    }

    Ok(())
}

async fn list_recursive(
    base: &std::path::Path,
    dir: &std::path::Path,
    depth: usize,
    max_depth: usize,
    entries: &mut Vec<String>,
    max_entries: usize,
) -> Result<(), ToolError> {
    if depth > max_depth || entries.len() >= max_entries {
        return Ok(());
    }

    let mut read_dir = tokio::fs::read_dir(dir)
        .await
        .map_err(|e| ToolError::Execution(format!("failed to read directory: {}", e)))?;

    let mut items = Vec::new();
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| ToolError::Execution(format!("failed to read entry: {}", e)))?
    {
        items.push(entry);
    }

    items.sort_by_key(|e| e.file_name());

    for entry in items {
        if entries.len() >= max_entries {
            entries.push("... (truncated)".to_string());
            return Ok(());
        }

        let metadata = entry.metadata().await;
        let rel_path = entry
            .path()
            .strip_prefix(base)
            .unwrap_or(&entry.path())
            .to_string_lossy()
            .to_string();

        let line = format_entry(&rel_path, &metadata);
        entries.push(line);

        if let Ok(ref meta) = metadata {
            if meta.is_dir() {
                Box::pin(list_recursive(
                    base,
                    &entry.path(),
                    depth + 1,
                    max_depth,
                    entries,
                    max_entries,
                ))
                .await?;
            }
        }
    }

    Ok(())
}

fn format_entry(name: &str, metadata: &Result<std::fs::Metadata, std::io::Error>) -> String {
    match metadata {
        Ok(meta) => {
            let kind = if meta.is_dir() {
                "dir"
            } else if meta.is_symlink() {
                "symlink"
            } else {
                "file"
            };
            let size = meta.len();
            let size_str = format_size(size);
            format!("[{}] {} ({})", kind, name, size_str)
        }
        Err(_) => format!("[?] {}", name),
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
