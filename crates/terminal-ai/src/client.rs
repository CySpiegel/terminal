use crate::config::AiConfig;
use crate::conversation::{ChatMessage, Role, ToolCall, ToolDefinition};
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// OpenAI-compatible LLM client.
///
/// Works with any endpoint that implements the OpenAI chat completions API:
/// OpenAI, Anthropic (via proxy), LiteLLM, Ollama, vLLM, text-generation-inference, etc.
pub struct LlmClient {
    http: reqwest::Client,
    config: AiConfig,
}

/// A streaming event from the LLM.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// A text token was generated.
    Token(String),
    /// The LLM wants to call a tool.
    ToolCallStart {
        id: String,
        name: String,
    },
    /// Arguments chunk for an in-progress tool call.
    ToolCallArgs(String),
    /// Response is complete.
    Done {
        usage: Option<Usage>,
    },
    /// An error occurred.
    Error(String),
}

/// Token usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Non-streaming completion response.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Usage,
    pub finish_reason: String,
}

impl LlmClient {
    /// Create a new LLM client with the given configuration.
    pub fn new(config: AiConfig) -> Result<Self, ClientError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(ref key) = config.api_key {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", key))
                    .map_err(|_| ClientError::InvalidApiKey)?,
            );
        }

        // Add custom headers
        for (k, v) in &config.custom_headers {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                headers.insert(name, val);
            }
        }

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| ClientError::HttpClient(e.to_string()))?;

        Ok(Self { http, config })
    }

    /// Send a chat completion request (non-streaming).
    pub async fn complete(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<CompletionResponse, ClientError> {
        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "stream": false,
        });

        if !tools.is_empty() {
            body["tools"] = serde_json::to_value(tools).unwrap_or_default();
            body["tool_choice"] = serde_json::Value::String(self.config.tool_choice.clone());
        }

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ClientError::Request(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ClientError::Parse(e.to_string()))?;

        let choice = &data["choices"][0];
        let message = &choice["message"];

        let content = message["content"].as_str().map(|s| s.to_string());

        let tool_calls: Vec<ToolCall> = message
            .get("tool_calls")
            .and_then(|tc| serde_json::from_value(tc.clone()).ok())
            .unwrap_or_default();

        let usage = Usage {
            prompt_tokens: data["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: data["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: data["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
        };

        let finish_reason = choice["finish_reason"]
            .as_str()
            .unwrap_or("stop")
            .to_string();

        Ok(CompletionResponse {
            content,
            tool_calls,
            usage,
            finish_reason,
        })
    }

    /// Send a streaming chat completion request.
    ///
    /// Returns a channel receiver that yields `StreamEvent`s as they arrive.
    pub async fn stream(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<mpsc::Receiver<StreamEvent>, ClientError> {
        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "stream": true,
        });

        if !tools.is_empty() {
            body["tools"] = serde_json::to_value(tools).unwrap_or_default();
            body["tool_choice"] = serde_json::Value::String(self.config.tool_choice.clone());
        }

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ClientError::Request(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let (tx, rx) = mpsc::channel(256);

        // Spawn a task to process the SSE stream
        let byte_stream = resp.bytes_stream();
        tokio::spawn(async move {
            let mut stream = eventsource_stream::Eventsource::new(byte_stream);
            while let Some(event) = stream.next().await {
                match event {
                    Ok(ev) => {
                        let data = ev.data;
                        if data == "[DONE]" {
                            let _ = tx.send(StreamEvent::Done { usage: None }).await;
                            break;
                        }

                        if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&data) {
                            let delta = &chunk["choices"][0]["delta"];

                            // Text content
                            if let Some(content) = delta["content"].as_str() {
                                if !content.is_empty() {
                                    let _ = tx.send(StreamEvent::Token(content.to_string())).await;
                                }
                            }

                            // Tool calls
                            if let Some(tool_calls) = delta.get("tool_calls") {
                                if let Some(arr) = tool_calls.as_array() {
                                    for tc in arr {
                                        if let Some(func) = tc.get("function") {
                                            if let Some(name) = func["name"].as_str() {
                                                let id = tc["id"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string();
                                                let _ = tx
                                                    .send(StreamEvent::ToolCallStart {
                                                        id,
                                                        name: name.to_string(),
                                                    })
                                                    .await;
                                            }
                                            if let Some(args) = func["arguments"].as_str() {
                                                if !args.is_empty() {
                                                    let _ = tx
                                                        .send(StreamEvent::ToolCallArgs(
                                                            args.to_string(),
                                                        ))
                                                        .await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Check for finish
                            if let Some(reason) = chunk["choices"][0]["finish_reason"].as_str() {
                                if reason == "stop" || reason == "tool_calls" {
                                    let usage = chunk.get("usage").and_then(|u| {
                                        Some(Usage {
                                            prompt_tokens: u["prompt_tokens"].as_u64()? as u32,
                                            completion_tokens: u["completion_tokens"].as_u64()?
                                                as u32,
                                            total_tokens: u["total_tokens"].as_u64()? as u32,
                                        })
                                    });
                                    let _ = tx.send(StreamEvent::Done { usage }).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Get current configuration.
    pub fn config(&self) -> &AiConfig {
        &self.config
    }

    /// Update the model.
    pub fn set_model(&mut self, model: &str) {
        self.config.model = model.to_string();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("invalid API key")]
    InvalidApiKey,
    #[error("HTTP client error: {0}")]
    HttpClient(String),
    #[error("request failed: {0}")]
    Request(String),
    #[error("API error (HTTP {status}): {body}")]
    Api { status: u16, body: String },
    #[error("failed to parse response: {0}")]
    Parse(String),
}
