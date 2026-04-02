use crate::tools::PermissionCategory;
use std::collections::HashSet;

/// Manages tool permissions.
///
/// Controls which tools the AI agent is allowed to execute.
/// Can be configured to auto-approve certain categories or
/// require explicit user approval.
pub struct PermissionManager {
    /// Tools that are always allowed.
    allowed: HashSet<String>,
    /// Tools that are always denied.
    denied: HashSet<String>,
    /// Categories that are auto-approved.
    auto_approve_categories: HashSet<String>,
    /// Default mode: "prompt" (ask user), "auto" (allow all), "deny" (block all).
    default_mode: PermissionMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionMode {
    /// Ask the user before executing (default).
    Prompt,
    /// Auto-approve all tool calls.
    Auto,
    /// Deny all tool calls unless explicitly allowed.
    Deny,
}

impl PermissionManager {
    /// Create a new permission manager with the given default mode.
    pub fn new(mode: PermissionMode) -> Self {
        let mut auto_approve = HashSet::new();
        // Read-only tools are always safe
        if mode == PermissionMode::Auto {
            auto_approve.insert("ReadOnly".to_string());
            auto_approve.insert("Write".to_string());
            auto_approve.insert("Execute".to_string());
            auto_approve.insert("Network".to_string());
        } else {
            // In prompt mode, at least allow read-only by default
            auto_approve.insert("ReadOnly".to_string());
        }

        Self {
            allowed: HashSet::new(),
            denied: HashSet::new(),
            auto_approve_categories: auto_approve,
            default_mode: mode,
        }
    }

    /// Check if a tool is allowed to execute.
    pub fn is_allowed(&self, tool_name: &str) -> bool {
        if self.denied.contains(tool_name) {
            return false;
        }
        if self.allowed.contains(tool_name) {
            return true;
        }
        match self.default_mode {
            PermissionMode::Auto => true,
            PermissionMode::Deny => false,
            PermissionMode::Prompt => {
                // In prompt mode, we'd normally ask the user.
                // For now, auto-approve read-only tools.
                true // TODO: integrate with UI for interactive permission prompts
            }
        }
    }

    /// Explicitly allow a tool.
    pub fn allow(&mut self, tool_name: &str) {
        self.allowed.insert(tool_name.to_string());
        self.denied.remove(tool_name);
    }

    /// Explicitly deny a tool.
    pub fn deny(&mut self, tool_name: &str) {
        self.denied.insert(tool_name.to_string());
        self.allowed.remove(tool_name);
    }

    /// Auto-approve an entire permission category.
    pub fn auto_approve_category(&mut self, category: &PermissionCategory) {
        let cat_str = match category {
            PermissionCategory::ReadOnly => "ReadOnly",
            PermissionCategory::Write => "Write",
            PermissionCategory::Execute => "Execute",
            PermissionCategory::Network => "Network",
        };
        self.auto_approve_categories.insert(cat_str.to_string());
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new(PermissionMode::Prompt)
    }
}
