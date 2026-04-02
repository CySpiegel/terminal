use crate::client::{LlmClient, StreamEvent};
use crate::conversation::{Conversation, ToolCall, ToolDefinition};
use crate::tools::ToolRegistry;
use crate::permission::PermissionManager;
use tokio::sync::mpsc;

/// The AI agent loop.
///
/// Orchestrates the cycle: user input → LLM → tool calls → tool results → LLM → response.
/// Runs until the LLM produces a final text response or hits the iteration limit.
pub struct Agent {
    client: LlmClient,
    conversation: Conversation,
    tools: ToolRegistry,
    permissions: PermissionManager,
    max_iterations: u32,
}

/// Events emitted by the agent during execution.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Streaming text token from the LLM.
    Token(String),
    /// The LLM is requesting a tool call.
    ToolCallStart { name: String, id: String },
    /// Tool execution result.
    ToolResult { name: String, result: String, success: bool },
    /// A complete response has been assembled.
    Response(String),
    /// Agent loop iteration count.
    Iteration(u32),
    /// Agent finished.
    Done,
    /// Error occurred.
    Error(String),
}

impl Agent {
    pub fn new(
        client: LlmClient,
        system_prompt: &str,
        tools: ToolRegistry,
        permissions: PermissionManager,
        max_iterations: u32,
    ) -> Self {
        Self {
            client,
            conversation: Conversation::new(system_prompt),
            tools,
            permissions,
            max_iterations,
        }
    }

    /// Submit a user message and run the agent loop.
    ///
    /// Returns a channel that streams `AgentEvent`s as the agent works.
    pub async fn submit(
        &mut self,
        user_message: &str,
    ) -> Result<mpsc::Receiver<AgentEvent>, crate::client::ClientError> {
        self.conversation.add_user(user_message);

        let (tx, rx) = mpsc::channel(256);
        let tool_defs = self.tools.definitions();

        // Agent loop
        let mut iteration = 0u32;
        loop {
            iteration += 1;
            if iteration > self.max_iterations {
                let _ = tx.send(AgentEvent::Error(format!(
                    "Agent loop exceeded {} iterations",
                    self.max_iterations
                ))).await;
                break;
            }

            let _ = tx.send(AgentEvent::Iteration(iteration)).await;

            // Call LLM
            let response = self
                .client
                .complete(&self.conversation.messages(), &tool_defs)
                .await?;

            // If there are tool calls, execute them
            if !response.tool_calls.is_empty() {
                self.conversation
                    .add_assistant(response.content.clone(), response.tool_calls.clone());

                for tool_call in &response.tool_calls {
                    let _ = tx
                        .send(AgentEvent::ToolCallStart {
                            name: tool_call.function.name.clone(),
                            id: tool_call.id.clone(),
                        })
                        .await;

                    // Check permissions
                    if !self.permissions.is_allowed(&tool_call.function.name) {
                        let result = format!(
                            "Permission denied for tool '{}'. User has not granted access.",
                            tool_call.function.name
                        );
                        self.conversation.add_tool_result(&tool_call.id, &result);
                        let _ = tx
                            .send(AgentEvent::ToolResult {
                                name: tool_call.function.name.clone(),
                                result,
                                success: false,
                            })
                            .await;
                        continue;
                    }

                    // Execute tool
                    let result = self
                        .tools
                        .execute(
                            &tool_call.function.name,
                            &tool_call.function.arguments,
                        )
                        .await;

                    let (result_str, success) = match result {
                        Ok(output) => (output, true),
                        Err(e) => (format!("Error: {}", e), false),
                    };

                    self.conversation.add_tool_result(&tool_call.id, &result_str);
                    let _ = tx
                        .send(AgentEvent::ToolResult {
                            name: tool_call.function.name.clone(),
                            result: result_str,
                            success,
                        })
                        .await;
                }

                // Continue the loop — LLM needs to process tool results
                continue;
            }

            // No tool calls — this is the final response
            if let Some(ref content) = response.content {
                let _ = tx.send(AgentEvent::Response(content.clone())).await;
            }

            self.conversation
                .add_assistant(response.content, Vec::new());
            break;
        }

        let _ = tx.send(AgentEvent::Done).await;
        Ok(rx)
    }

    /// Clear the conversation history.
    pub fn reset(&mut self) {
        self.conversation.clear();
    }

    /// Get a reference to the conversation.
    pub fn conversation(&self) -> &Conversation {
        &self.conversation
    }
}
