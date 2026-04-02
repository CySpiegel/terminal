use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "notebook_edit".to_string(),
        description: "Edit a cell in a Jupyter notebook (.ipynb) file. Replaces the source content of a cell at the given index.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute path to the .ipynb file"
                },
                "cell_index": {
                    "type": "integer",
                    "description": "Zero-based index of the cell to edit"
                },
                "new_source": {
                    "type": "string",
                    "description": "The new source content for the cell"
                }
            },
            "required": ["path", "cell_index", "new_source"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let path = args["path"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'path'".to_string()))?;

    let cell_index = args["cell_index"]
        .as_u64()
        .ok_or_else(|| ToolError::InvalidArgs("missing or invalid 'cell_index'".to_string()))?
        as usize;

    let new_source = args["new_source"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'new_source'".to_string()))?;

    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| ToolError::Execution(format!("failed to read {}: {}", path, e)))?;

    let mut notebook: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| ToolError::Execution(format!("invalid notebook JSON: {}", e)))?;

    let cells = notebook["cells"]
        .as_array_mut()
        .ok_or_else(|| ToolError::Execution("notebook has no 'cells' array".to_string()))?;

    if cell_index >= cells.len() {
        return Err(ToolError::Execution(format!(
            "cell_index {} is out of range (notebook has {} cells)",
            cell_index,
            cells.len()
        )));
    }

    // Convert new_source into the notebook's source format (array of lines)
    let source_lines: Vec<serde_json::Value> = new_source
        .split('\n')
        .enumerate()
        .map(|(i, line)| {
            let total = new_source.split('\n').count();
            if i < total - 1 {
                serde_json::Value::String(format!("{}\n", line))
            } else {
                serde_json::Value::String(line.to_string())
            }
        })
        .collect();

    cells[cell_index]["source"] = serde_json::Value::Array(source_lines);

    let output = serde_json::to_string_pretty(&notebook)
        .map_err(|e| ToolError::Execution(format!("failed to serialize notebook: {}", e)))?;

    tokio::fs::write(path, &output)
        .await
        .map_err(|e| ToolError::Execution(format!("failed to write {}: {}", path, e)))?;

    Ok(format!(
        "Successfully edited cell {} in {}",
        cell_index, path
    ))
}
