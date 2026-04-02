use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "notebook_read".to_string(),
        description: "Read a Jupyter notebook (.ipynb) file. Parses the JSON structure and returns cells with their type (code/markdown), source, and outputs.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute path to the .ipynb file"
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

    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| ToolError::Execution(format!("failed to read {}: {}", path, e)))?;

    let notebook: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| ToolError::Execution(format!("invalid notebook JSON: {}", e)))?;

    let cells = notebook["cells"]
        .as_array()
        .ok_or_else(|| ToolError::Execution("notebook has no 'cells' array".to_string()))?;

    let mut result = Vec::new();

    for (i, cell) in cells.iter().enumerate() {
        let cell_type = cell["cell_type"].as_str().unwrap_or("unknown");
        let source = extract_source(&cell["source"]);

        result.push(format!("--- Cell {} [{}] ---", i, cell_type));
        result.push(source);

        // Extract outputs for code cells
        if cell_type == "code" {
            if let Some(outputs) = cell["outputs"].as_array() {
                for output in outputs {
                    let output_text = extract_output(output);
                    if !output_text.is_empty() {
                        result.push(format!("[Output]:\n{}", output_text));
                    }
                }
            }
        }

        result.push(String::new());
    }

    Ok(result.join("\n"))
}

/// Extract source text from a cell's "source" field (can be string or array of strings).
fn extract_source(source: &serde_json::Value) -> String {
    match source {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    }
}

/// Extract readable output from a cell output object.
fn extract_output(output: &serde_json::Value) -> String {
    let output_type = output["output_type"].as_str().unwrap_or("");

    match output_type {
        "stream" => extract_source(&output["text"]),
        "execute_result" | "display_data" => {
            if let Some(data) = output["data"].as_object() {
                // Prefer text/plain, fall back to other types
                if let Some(text) = data.get("text/plain") {
                    return extract_source(text);
                }
                if let Some(html) = data.get("text/html") {
                    return format!("[HTML output]: {}", extract_source(html));
                }
                if data.contains_key("image/png") {
                    return "[Image output: PNG]".to_string();
                }
                format!("[Output with types: {:?}]", data.keys().collect::<Vec<_>>())
            } else {
                String::new()
            }
        }
        "error" => {
            let ename = output["ename"].as_str().unwrap_or("Error");
            let evalue = output["evalue"].as_str().unwrap_or("");
            format!("[Error] {}: {}", ename, evalue)
        }
        _ => String::new(),
    }
}
