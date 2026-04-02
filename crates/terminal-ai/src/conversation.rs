use serde::{Deserialize, Serialize};

/// A message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tool_calls: Vec<ToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Message role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A tool call requested by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

/// Function call details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Tool definition in OpenAI function-calling format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

/// Function definition with JSON Schema parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Conversation history with token tracking.
pub struct Conversation {
    messages: Vec<ChatMessage>,
    system_prompt: String,
}

impl Conversation {
    pub fn new(system_prompt: &str) -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: system_prompt.to_string(),
        }
    }

    /// Get all messages including the system prompt.
    pub fn messages(&self) -> Vec<ChatMessage> {
        let mut msgs = vec![ChatMessage {
            role: Role::System,
            content: Some(self.system_prompt.clone()),
            name: None,
            tool_calls: Vec::new(),
            tool_call_id: None,
        }];
        msgs.extend(self.messages.clone());
        msgs
    }

    /// Add a user message.
    pub fn add_user(&mut self, content: &str) {
        self.messages.push(ChatMessage {
            role: Role::User,
            content: Some(content.to_string()),
            name: None,
            tool_calls: Vec::new(),
            tool_call_id: None,
        });
    }

    /// Add an assistant message.
    pub fn add_assistant(&mut self, content: Option<String>, tool_calls: Vec<ToolCall>) {
        self.messages.push(ChatMessage {
            role: Role::Assistant,
            content,
            name: None,
            tool_calls,
            tool_call_id: None,
        });
    }

    /// Add a tool result message.
    pub fn add_tool_result(&mut self, tool_call_id: &str, content: &str) {
        self.messages.push(ChatMessage {
            role: Role::Tool,
            content: Some(content.to_string()),
            name: None,
            tool_calls: Vec::new(),
            tool_call_id: Some(tool_call_id.to_string()),
        });
    }

    /// Clear conversation history (keeps system prompt).
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get the number of messages (excluding system prompt).
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if conversation is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}
