use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;

/// Hook event types that can be listened to.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookEvent {
    BeforeMessage,
    AfterMessage,
    BeforeToolCall,
    AfterToolCall,
    OnCompact,
    SessionStart,
    SessionEnd,
    FileModified,
    CommandExecuted,
    Custom(String),
}

/// A hook configuration — runs a shell command when an event fires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub event: HookEvent,
    pub command: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_timeout() -> u64 { 30_000 }
fn default_true() -> bool { true }

/// Context passed to hooks via environment variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    pub event: HookEvent,
    pub data: serde_json::Value,
    pub timestamp: String,
    pub session_id: String,
}

/// Result of a hook execution.
#[derive(Debug)]
pub struct HookResult {
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
}

/// Manages lifecycle hooks.
pub struct HookManager {
    hooks: HashMap<HookEvent, Vec<HookConfig>>,
    session_id: String,
}

impl HookManager {
    pub fn new(session_id: &str) -> Self {
        Self {
            hooks: HashMap::new(),
            session_id: session_id.to_string(),
        }
    }

    /// Register a hook for an event.
    pub fn register(&mut self, config: HookConfig) {
        self.hooks
            .entry(config.event.clone())
            .or_default()
            .push(config);
    }

    /// Unregister a hook by event and index.
    pub fn unregister(&mut self, event: &HookEvent, index: usize) {
        if let Some(hooks) = self.hooks.get_mut(event) {
            if index < hooks.len() {
                hooks.remove(index);
            }
        }
    }

    /// Fire all hooks registered for an event.
    pub async fn fire(&self, event: HookEvent, data: serde_json::Value) -> Vec<HookResult> {
        let hooks = match self.hooks.get(&event) {
            Some(hooks) => hooks,
            None => return Vec::new(),
        };

        let mut results = Vec::new();

        for hook in hooks {
            if !hook.enabled {
                continue;
            }

            let context = HookContext {
                event: event.clone(),
                data: data.clone(),
                timestamp: chrono::Local::now().to_rfc3339(),
                session_id: self.session_id.clone(),
            };

            let context_json = serde_json::to_string(&context).unwrap_or_default();
            let start = std::time::Instant::now();

            let result = tokio::time::timeout(
                std::time::Duration::from_millis(hook.timeout_ms),
                Command::new("sh")
                    .arg("-c")
                    .arg(&hook.command)
                    .env("HOOK_CONTEXT", &context_json)
                    .env("HOOK_EVENT", serde_json::to_string(&event).unwrap_or_default())
                    .output(),
            )
            .await;

            let duration_ms = start.elapsed().as_millis() as u64;

            match result {
                Ok(Ok(output)) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let combined = if stderr.is_empty() {
                        stdout
                    } else {
                        format!("{}\n{}", stdout, stderr)
                    };

                    results.push(HookResult {
                        success: output.status.success(),
                        output: combined,
                        duration_ms,
                    });
                }
                Ok(Err(e)) => {
                    results.push(HookResult {
                        success: false,
                        output: format!("Hook execution error: {}", e),
                        duration_ms,
                    });
                }
                Err(_) => {
                    results.push(HookResult {
                        success: false,
                        output: format!("Hook timed out after {}ms", hook.timeout_ms),
                        duration_ms,
                    });
                }
            }
        }

        results
    }

    /// List all registered hooks.
    pub fn list(&self) -> Vec<(&HookEvent, &Vec<HookConfig>)> {
        self.hooks.iter().collect()
    }
}
