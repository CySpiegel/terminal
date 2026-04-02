use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "web_fetch".to_string(),
        description: "Fetch the content of a URL. Returns the page text with HTML tags stripped for readability.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch"
                },
                "max_length": {
                    "type": "integer",
                    "description": "Maximum number of characters to return (default: 50000)"
                }
            },
            "required": ["url"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let url = args["url"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'url'".to_string()))?;

    let max_length = args["max_length"].as_u64().unwrap_or(50_000) as usize;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| ToolError::Execution(format!("failed to create HTTP client: {}", e)))?;

    let response = client
        .get(url)
        .header("User-Agent", "terminal-ai/1.0")
        .send()
        .await
        .map_err(|e| ToolError::Execution(format!("fetch failed: {}", e)))?;

    let status = response.status();
    if !status.is_success() {
        return Err(ToolError::Execution(format!(
            "HTTP {} for {}",
            status, url
        )));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let body = response
        .text()
        .await
        .map_err(|e| ToolError::Execution(format!("failed to read response body: {}", e)))?;

    // Strip HTML tags if the content appears to be HTML
    let text = if content_type.contains("html") || body.trim_start().starts_with('<') {
        strip_html(&body)
    } else {
        body
    };

    let mut result = text;
    if result.len() > max_length {
        result.truncate(max_length);
        result.push_str("\n... (content truncated)");
    }

    Ok(result)
}

/// Simple HTML tag stripper that extracts readable text.
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len() / 2);
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut last_was_whitespace = false;

    let lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = lower.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        if in_script {
            // Look for </script>
            if i + 9 <= lower_chars.len()
                && lower_chars[i..i + 9].iter().collect::<String>() == "</script>"
            {
                in_script = false;
                i += 9;
                continue;
            }
            i += 1;
            continue;
        }

        if in_style {
            // Look for </style>
            if i + 8 <= lower_chars.len()
                && lower_chars[i..i + 8].iter().collect::<String>() == "</style>"
            {
                in_style = false;
                i += 8;
                continue;
            }
            i += 1;
            continue;
        }

        if chars[i] == '<' {
            in_tag = true;
            // Check for <script or <style
            if i + 7 <= lower_chars.len()
                && lower_chars[i..i + 7].iter().collect::<String>() == "<script"
            {
                in_script = true;
            } else if i + 6 <= lower_chars.len()
                && lower_chars[i..i + 6].iter().collect::<String>() == "<style"
            {
                in_style = true;
            }
            i += 1;
            continue;
        }

        if chars[i] == '>' && in_tag {
            in_tag = false;
            i += 1;
            continue;
        }

        if !in_tag {
            let ch = chars[i];
            if ch.is_whitespace() {
                if !last_was_whitespace {
                    result.push(' ');
                    last_was_whitespace = true;
                }
            } else {
                result.push(ch);
                last_was_whitespace = false;
            }
        }

        i += 1;
    }

    // Decode common HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}
