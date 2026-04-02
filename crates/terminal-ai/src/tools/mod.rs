//! Modular tool system for the AI agent.
//!
//! Each tool is self-contained with:
//! - A JSON Schema definition (for the LLM)
//! - An async execute function
//! - A permission category
//!
//! Inspired by Claude Code's 40+ tool architecture.

pub mod file_read;
pub mod file_write;
pub mod file_edit;
pub mod bash;
pub mod grep;
pub mod glob;

use crate::conversation::{FunctionDefinition, ToolDefinition};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Result type for tool execution.
pub type ToolResult = Result<String, ToolError>;

/// Async tool execution function signature.
pub type ToolFn = Box<
    dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>>
        + Send
        + Sync,
>;

/// Tool registration and execution.
pub struct ToolRegistry {
    tools: HashMap<String, RegisteredTool>,
}

struct RegisteredTool {
    definition: ToolDefinition,
    execute: ToolFn,
    permission_category: PermissionCategory,
}

/// Permission categories for tools.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionCategory {
    /// Read-only operations (file read, grep, glob).
    ReadOnly,
    /// Write operations (file write, file edit).
    Write,
    /// Shell execution (bash commands).
    Execute,
    /// Network operations (web fetch, API calls).
    Network,
}

impl ToolRegistry {
    /// Create a new registry with all built-in tools registered.
    pub fn with_builtins() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Register all built-in tools
        registry.register_builtin("file_read", file_read::definition(), file_read::execute, PermissionCategory::ReadOnly);
        registry.register_builtin("file_write", file_write::definition(), file_write::execute, PermissionCategory::Write);
        registry.register_builtin("file_edit", file_edit::definition(), file_edit::execute, PermissionCategory::Write);
        registry.register_builtin("bash", bash::definition(), bash::execute, PermissionCategory::Execute);
        registry.register_builtin("grep", grep::definition(), grep::execute, PermissionCategory::ReadOnly);
        registry.register_builtin("glob_search", glob::definition(), glob::execute, PermissionCategory::ReadOnly);

        registry
    }

    fn register_builtin<F, Fut>(
        &mut self,
        name: &str,
        func_def: FunctionDefinition,
        execute_fn: F,
        category: PermissionCategory,
    ) where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ToolResult> + Send + 'static,
    {
        let definition = ToolDefinition {
            tool_type: "function".to_string(),
            function: func_def,
        };

        self.tools.insert(
            name.to_string(),
            RegisteredTool {
                definition,
                execute: Box::new(move |args| Box::pin(execute_fn(args))),
                permission_category: category,
            },
        );
    }

    /// Register a custom tool at runtime.
    pub fn register_custom(
        &mut self,
        name: &str,
        definition: FunctionDefinition,
        execute: ToolFn,
        category: PermissionCategory,
    ) {
        self.tools.insert(
            name.to_string(),
            RegisteredTool {
                definition: ToolDefinition {
                    tool_type: "function".to_string(),
                    function: definition,
                },
                execute,
                permission_category: category,
            },
        );
    }

    /// Get tool definitions for the LLM API call.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition.clone()).collect()
    }

    /// Execute a tool by name with JSON arguments.
    pub async fn execute(&self, name: &str, args_json: &str) -> ToolResult {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;

        let args: serde_json::Value =
            serde_json::from_str(args_json).unwrap_or(serde_json::Value::Object(Default::default()));

        (tool.execute)(args).await
    }

    /// Get the permission category for a tool.
    pub fn permission_category(&self, name: &str) -> Option<&PermissionCategory> {
        self.tools.get(name).map(|t| &t.permission_category)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("tool not found: {0}")]
    NotFound(String),
    #[error("invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("execution failed: {0}")]
    Execution(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
