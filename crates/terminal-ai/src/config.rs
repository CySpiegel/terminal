use serde::{Deserialize, Serialize};

/// Configuration for the AI assistant.
///
/// Supports any OpenAI-compatible API endpoint — set `base_url` to your
/// LiteLLM proxy, Ollama instance, vLLM server, or any other compatible host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Base URL for the OpenAI-compatible API.
    /// Examples:
    ///   - "https://api.openai.com/v1" (OpenAI)
    ///   - "https://api.anthropic.com/v1" (Anthropic via proxy)
    ///   - "http://localhost:4000" (LiteLLM proxy)
    ///   - "http://localhost:11434/v1" (Ollama)
    ///   - "http://localhost:8000/v1" (vLLM)
    ///   - "http://your-server:8080/v1" (any self-hosted)
    pub base_url: String,

    /// API key (optional for local models).
    pub api_key: Option<String>,

    /// Model identifier.
    /// Examples: "gpt-4o", "claude-sonnet-4-20250514", "llama3.1:70b", "mistral-large"
    pub model: String,

    /// Maximum tokens in completion response.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Temperature for generation (0.0 = deterministic, 1.0 = creative).
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// System prompt prepended to all conversations.
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,

    /// Request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Whether to stream responses token-by-token.
    #[serde(default = "default_true")]
    pub stream: bool,

    /// Custom headers to send with API requests (e.g., for auth proxies).
    #[serde(default)]
    pub custom_headers: std::collections::HashMap<String, String>,

    /// Tool use mode: "auto", "required", or "none".
    #[serde(default = "default_tool_choice")]
    pub tool_choice: String,

    /// Maximum agent loop iterations (tool calls) before stopping.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
}

fn default_max_tokens() -> u32 { 4096 }
fn default_temperature() -> f32 { 0.0 }
fn default_timeout() -> u64 { 120 }
fn default_true() -> bool { true }
fn default_tool_choice() -> String { "auto".to_string() }
fn default_max_iterations() -> u32 { 50 }

fn default_system_prompt() -> String {
    "You are an AI coding assistant integrated into a GPU-rendered terminal emulator. \
     You have access to tools for reading, writing, and editing files, searching code, \
     running shell commands, and managing tasks. Be concise and helpful."
        .to_string()
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: None,
            model: "llama3.1:8b".to_string(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            system_prompt: default_system_prompt(),
            timeout_secs: default_timeout(),
            stream: true,
            custom_headers: std::collections::HashMap::new(),
            tool_choice: default_tool_choice(),
            max_iterations: default_max_iterations(),
        }
    }
}

impl AiConfig {
    /// Create config for a LiteLLM proxy.
    pub fn litellm(url: &str, model: &str, api_key: Option<&str>) -> Self {
        Self {
            base_url: url.to_string(),
            api_key: api_key.map(|s| s.to_string()),
            model: model.to_string(),
            ..Default::default()
        }
    }

    /// Create config for a local Ollama instance.
    pub fn ollama(model: &str) -> Self {
        Self {
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: None,
            model: model.to_string(),
            ..Default::default()
        }
    }

    /// Create config for a vLLM server.
    pub fn vllm(url: &str, model: &str) -> Self {
        Self {
            base_url: format!("{}/v1", url.trim_end_matches('/')),
            api_key: None,
            model: model.to_string(),
            ..Default::default()
        }
    }

    /// Create config for OpenAI.
    pub fn openai(api_key: &str, model: &str) -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: Some(api_key.to_string()),
            model: model.to_string(),
            ..Default::default()
        }
    }

    /// Create config for any custom endpoint.
    pub fn custom(base_url: &str, model: &str, api_key: Option<&str>) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.map(|s| s.to_string()),
            model: model.to_string(),
            ..Default::default()
        }
    }
}
