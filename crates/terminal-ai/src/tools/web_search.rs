use super::{ToolError, ToolResult};
use crate::conversation::FunctionDefinition;

pub fn definition() -> FunctionDefinition {
    FunctionDefinition {
        name: "web_search".to_string(),
        description: "Search the web for a given query and return results. Returns titles, URLs, and snippets.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default: 5)"
                }
            },
            "required": ["query"]
        }),
    }
}

pub async fn execute(args: serde_json::Value) -> ToolResult {
    let query = args["query"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgs("missing 'query'".to_string()))?;

    let max_results = args["max_results"].as_u64().unwrap_or(5) as usize;

    // NOTE: This uses DuckDuckGo's lite HTML interface for zero-API-key search.
    // For production use, swap this for a proper search API (Google Custom Search,
    // Brave Search API, SerpAPI, etc.) by setting an API key in config.
    let encoded_query = urlencoding::encode(query);
    let search_url = format!("https://lite.duckduckgo.com/lite/?q={}", encoded_query);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| ToolError::Execution(format!("failed to create HTTP client: {}", e)))?;

    let response = client
        .get(&search_url)
        .header("User-Agent", "terminal-ai/1.0")
        .send()
        .await
        .map_err(|e| ToolError::Execution(format!("search request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(ToolError::Execution(format!(
            "search returned HTTP {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| ToolError::Execution(format!("failed to read search response: {}", e)))?;

    let results = extract_search_results(&body, max_results);

    if results.is_empty() {
        Ok(format!("No results found for: {}", query))
    } else {
        Ok(results.join("\n\n"))
    }
}

/// Extract search results from DuckDuckGo lite HTML.
fn extract_search_results(html: &str, max: usize) -> Vec<String> {
    let mut results = Vec::new();

    // DuckDuckGo lite returns results as links in the page.
    // We extract <a> tags with class "result-link" or hrefs that look like results.
    let mut pos = 0;
    while results.len() < max {
        // Find next result link
        let link_start = match html[pos..].find("href=\"") {
            Some(idx) => pos + idx + 6,
            None => break,
        };

        let link_end = match html[link_start..].find('"') {
            Some(idx) => link_start + idx,
            None => break,
        };

        let url = &html[link_start..link_end];
        pos = link_end;

        // Skip internal DDG links and navigation
        if url.starts_with('/')
            || url.contains("duckduckgo.com")
            || url.contains("duck.co")
            || url.is_empty()
        {
            continue;
        }

        // Try to extract the link text (title)
        let title = if let Some(gt) = html[link_end..].find('>') {
            let text_start = link_end + gt + 1;
            if let Some(lt) = html[text_start..].find('<') {
                html[text_start..text_start + lt].trim().to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if title.is_empty() || url.len() < 10 {
            continue;
        }

        let entry = format!(
            "{}. {}\n   {}",
            results.len() + 1,
            title,
            url
        );
        results.push(entry);
    }

    results
}
