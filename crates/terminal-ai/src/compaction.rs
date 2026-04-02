//! Conversation compaction for managing context window usage.
//!
//! When conversations grow long, older messages can be summarized into
//! a single compact message to free up context window space while
//! preserving key information.

use crate::conversation::{ChatMessage, Role};

/// Configuration for conversation compaction.
#[derive(Debug, Clone)]
pub struct CompactionConfig {
    /// Number of recent messages to keep intact (not summarized).
    pub keep_recent: usize,
    /// Maximum number of tokens for the summary message.
    pub max_summary_tokens: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            keep_recent: 10,
            max_summary_tokens: 2048,
        }
    }
}

/// Estimate the number of tokens in a slice of messages.
///
/// Uses a rough heuristic of ~4 characters per token.
pub fn estimate_tokens(messages: &[ChatMessage]) -> usize {
    messages
        .iter()
        .map(|msg| {
            let content_len = msg.content.as_deref().unwrap_or("").len();
            let tool_calls_len: usize = msg
                .tool_calls
                .iter()
                .map(|tc| tc.function.name.len() + tc.function.arguments.len())
                .sum();
            // ~4 chars per token, plus overhead for role/metadata
            (content_len + tool_calls_len + 10) / 4
        })
        .sum()
}

/// Compact a conversation by summarizing older messages.
///
/// The system prompt (first message with `Role::System`) is always preserved.
/// The most recent `config.keep_recent` messages are kept intact.
/// All messages in between are summarized into a single user message
/// containing a text summary.
///
/// If the conversation is short enough that there is nothing to compact,
/// the original messages are returned unchanged.
pub fn compact_conversation(
    messages: &[ChatMessage],
    config: &CompactionConfig,
) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return Vec::new();
    }

    // Separate system prompt from the rest
    let (system_msgs, rest) = if messages.first().map(|m| &m.role) == Some(&Role::System) {
        (vec![messages[0].clone()], &messages[1..])
    } else {
        (Vec::new(), messages)
    };

    // If we have fewer messages than keep_recent, nothing to compact
    if rest.len() <= config.keep_recent {
        return messages.to_vec();
    }

    // Split into old (to summarize) and recent (to keep)
    let split_point = rest.len() - config.keep_recent;
    let old_messages = &rest[..split_point];
    let recent_messages = &rest[split_point..];

    // Build summary of old messages
    let summary = build_summary(old_messages, config.max_summary_tokens);

    let summary_message = ChatMessage {
        role: Role::User,
        content: Some(format!(
            "[Conversation Summary — {} earlier messages compacted]\n\n{}",
            old_messages.len(),
            summary,
        )),
        name: None,
        tool_calls: Vec::new(),
        tool_call_id: None,
    };

    // An assistant acknowledgment so the conversation alternation is valid
    let ack_message = ChatMessage {
        role: Role::Assistant,
        content: Some(
            "Understood. I have the context from the conversation summary above.".to_string(),
        ),
        name: None,
        tool_calls: Vec::new(),
        tool_call_id: None,
    };

    let mut result = system_msgs;
    result.push(summary_message);
    result.push(ack_message);
    result.extend_from_slice(recent_messages);
    result
}

/// Build a text summary from a list of messages.
///
/// Extracts the key content from each message, truncating to stay
/// within the token budget.
fn build_summary(messages: &[ChatMessage], max_tokens: usize) -> String {
    let max_chars = max_tokens * 4; // rough: 4 chars per token
    let mut summary = String::new();

    for msg in messages {
        let role_label = match msg.role {
            Role::User => "User",
            Role::Assistant => "Assistant",
            Role::Tool => "Tool",
            Role::System => "System",
        };

        if let Some(ref content) = msg.content {
            let line = if content.len() > 200 {
                format!("{}: {}...\n", role_label, &content[..200])
            } else {
                format!("{}: {}\n", role_label, content)
            };
            summary.push_str(&line);
        }

        for tc in &msg.tool_calls {
            summary.push_str(&format!("  -> Tool call: {}\n", tc.function.name));
        }

        if summary.len() >= max_chars {
            summary.truncate(max_chars);
            summary.push_str("\n... (truncated)");
            break;
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(role: Role, content: &str) -> ChatMessage {
        ChatMessage {
            role,
            content: Some(content.to_string()),
            name: None,
            tool_calls: Vec::new(),
            tool_call_id: None,
        }
    }

    #[test]
    fn test_estimate_tokens() {
        let msgs = vec![make_msg(Role::User, "Hello, world!")];
        let tokens = estimate_tokens(&msgs);
        // "Hello, world!" is 13 chars + 10 overhead = 23, / 4 = 5
        assert!(tokens > 0);
    }

    #[test]
    fn test_compact_short_conversation_unchanged() {
        let msgs = vec![
            make_msg(Role::System, "You are helpful."),
            make_msg(Role::User, "Hi"),
            make_msg(Role::Assistant, "Hello!"),
        ];
        let config = CompactionConfig {
            keep_recent: 10,
            max_summary_tokens: 2048,
        };
        let result = compact_conversation(&msgs, &config);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_compact_long_conversation() {
        let mut msgs = vec![make_msg(Role::System, "You are helpful.")];
        for i in 0..20 {
            msgs.push(make_msg(Role::User, &format!("Question {}", i)));
            msgs.push(make_msg(Role::Assistant, &format!("Answer {}", i)));
        }

        let config = CompactionConfig {
            keep_recent: 4,
            max_summary_tokens: 2048,
        };
        let result = compact_conversation(&msgs, &config);

        // Should have: system + summary + ack + 4 recent = 7
        assert_eq!(result.len(), 7);
        assert_eq!(result[0].role, Role::System);
        assert!(result[1]
            .content
            .as_ref()
            .unwrap()
            .contains("Conversation Summary"));
        assert_eq!(result[2].role, Role::Assistant);
    }

    #[test]
    fn test_compact_empty() {
        let result = compact_conversation(&[], &CompactionConfig::default());
        assert!(result.is_empty());
    }

    #[test]
    fn test_compact_no_system_prompt() {
        let mut msgs = Vec::new();
        for i in 0..20 {
            msgs.push(make_msg(Role::User, &format!("Q{}", i)));
            msgs.push(make_msg(Role::Assistant, &format!("A{}", i)));
        }

        let config = CompactionConfig {
            keep_recent: 4,
            max_summary_tokens: 2048,
        };
        let result = compact_conversation(&msgs, &config);

        // No system prompt, so: summary + ack + 4 recent = 6
        assert_eq!(result.len(), 6);
        assert!(result[0]
            .content
            .as_ref()
            .unwrap()
            .contains("Conversation Summary"));
    }
}
