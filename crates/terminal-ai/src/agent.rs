use crate::client::LlmClient;
use crate::conversation::{Conversation, ToolDefinition};
use crate::cost::CostTracker;
use crate::hooks::{HookEvent, HookManager};
use crate::mcp::McpManager;
use crate::multi_agent::AgentCoordinator;
use crate::permission::PermissionManager;
use crate::tools::ToolRegistry;
use tokio::sync::mpsc;

/// The AI agent loop.
///
/// Orchestrates the cycle: user input → LLM → tool calls → tool results → LLM → response.
/// Runs until the LLM produces a final text response or hits the iteration limit.
///
/// Supports:
/// - Local tools (file read/write/edit, bash, grep, glob, etc.)
/// - MCP tools (from connected MCP servers)
/// - Sub-agent spawning (via the "agent" tool)
/// - Hooks (lifecycle events fired before/after messages and tool calls)
/// - Cost tracking (token usage and estimated cost)
pub struct Agent {
    client: LlmClient,
    conversation: Conversation,
    tools: ToolRegistry,
    permissions: PermissionManager,
    max_iterations: u32,
    mcp: Option<McpManager>,
    hooks: Option<HookManager>,
    cost_tracker: CostTracker,
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
    /// A sub-agent was spawned.
    SubAgentSpawned { role: String, task: String },
    /// A sub-agent completed.
    SubAgentResult { role: String, output: String, success: bool },
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
            mcp: None,
            hooks: None,
            cost_tracker: CostTracker::new(),
        }
    }

    /// Attach an MCP manager for external tool servers.
    pub fn with_mcp(mut self, mcp: McpManager) -> Self {
        self.mcp = Some(mcp);
        self
    }

    /// Attach a hook manager for lifecycle events.
    pub fn with_hooks(mut self, hooks: HookManager) -> Self {
        self.hooks = Some(hooks);
        self
    }

    /// Submit a user message and run the agent loop.
    ///
    /// Returns a channel that streams `AgentEvent`s as the agent works.
    pub async fn submit(
        &mut self,
        user_message: &str,
    ) -> Result<mpsc::Receiver<AgentEvent>, crate::client::ClientError> {
        self.conversation.add_user(user_message);

        // Fire BeforeMessage hook
        if let Some(ref hooks) = self.hooks {
            hooks.fire(
                HookEvent::BeforeMessage,
                serde_json::json!({"message": user_message}),
            ).await;
        }

        let (tx, rx) = mpsc::channel(256);

        // Build tool definitions: local tools + MCP tools
        let mut tool_defs = self.tools.definitions();
        if let Some(ref mcp) = self.mcp {
            tool_defs.extend(mcp.all_tools());
        }

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

            // Track cost
            self.cost_tracker.record(
                &self.client.config().model,
                response.usage.prompt_tokens as u64,
                response.usage.completion_tokens as u64,
            );

            // If there are tool calls, execute them
            if !response.tool_calls.is_empty() {
                self.conversation
                    .add_assistant(response.content.clone(), response.tool_calls.clone());

                for tool_call in &response.tool_calls {
                    let tool_name = &tool_call.function.name;

                    let _ = tx
                        .send(AgentEvent::ToolCallStart {
                            name: tool_name.clone(),
                            id: tool_call.id.clone(),
                        })
                        .await;

                    // Fire BeforeToolCall hook
                    if let Some(ref hooks) = self.hooks {
                        hooks.fire(
                            HookEvent::BeforeToolCall,
                            serde_json::json!({
                                "tool": tool_name,
                                "arguments": tool_call.function.arguments
                            }),
                        ).await;
                    }

                    // Check permissions
                    if !self.permissions.is_allowed(tool_name) {
                        let result = format!(
                            "Permission denied for tool '{}'. User has not granted access.",
                            tool_name
                        );
                        self.conversation.add_tool_result(&tool_call.id, &result);
                        let _ = tx
                            .send(AgentEvent::ToolResult {
                                name: tool_name.clone(),
                                result,
                                success: false,
                            })
                            .await;
                        continue;
                    }

                    // Route tool call to the right handler
                    let (result_str, success) = if tool_name == "agent" {
                        // Handle sub-agent spawning
                        self.handle_agent_tool(&tool_call.function.arguments, &tx).await
                    } else if tool_name.starts_with("mcp_") {
                        // Route to MCP server
                        self.handle_mcp_tool(tool_name, &tool_call.function.arguments).await
                    } else {
                        // Execute local tool
                        match self.tools.execute(tool_name, &tool_call.function.arguments).await {
                            Ok(output) => (output, true),
                            Err(e) => (format!("Error: {}", e), false),
                        }
                    };

                    self.conversation.add_tool_result(&tool_call.id, &result_str);

                    // Fire AfterToolCall hook
                    if let Some(ref hooks) = self.hooks {
                        hooks.fire(
                            HookEvent::AfterToolCall,
                            serde_json::json!({
                                "tool": tool_name,
                                "result": &result_str,
                                "success": success
                            }),
                        ).await;
                    }

                    // Fire FileModified hook if a write tool was used
                    if success && (tool_name == "file_write" || tool_name == "file_edit" || tool_name == "file_insert") {
                        if let Some(ref hooks) = self.hooks {
                            hooks.fire(
                                HookEvent::FileModified,
                                serde_json::json!({"tool": tool_name}),
                            ).await;
                        }
                    }

                    let _ = tx
                        .send(AgentEvent::ToolResult {
                            name: tool_name.clone(),
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

        // Fire AfterMessage hook
        if let Some(ref hooks) = self.hooks {
            hooks.fire(
                HookEvent::AfterMessage,
                serde_json::json!({"iteration_count": iteration}),
            ).await;
        }

        let _ = tx.send(AgentEvent::Done).await;
        Ok(rx)
    }

    /// Handle the "agent" tool call by spawning a sub-agent.
    async fn handle_agent_tool(
        &self,
        args_json: &str,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> (String, bool) {
        let args: serde_json::Value = match serde_json::from_str(args_json) {
            Ok(v) => v,
            Err(e) => return (format!("Invalid agent args: {}", e), false),
        };

        let task = match args["task"].as_str() {
            Some(t) => t,
            None => return ("Missing 'task' argument".to_string(), false),
        };
        let role = args["role"].as_str().unwrap_or("general");

        let _ = tx.send(AgentEvent::SubAgentSpawned {
            role: role.to_string(),
            task: task.to_string(),
        }).await;

        // Spawn sub-agent via coordinator
        let config = self.client.config().clone();
        let mut coordinator = AgentCoordinator::new();
        coordinator.spawn_agent(role, task, config);
        coordinator.start_all();
        let results = coordinator.wait_all().await;

        if let Some(result) = results.into_iter().next() {
            let _ = tx.send(AgentEvent::SubAgentResult {
                role: result.role.clone(),
                output: result.output.clone(),
                success: result.success,
            }).await;
            (result.output, result.success)
        } else {
            ("Sub-agent produced no result".to_string(), false)
        }
    }

    /// Handle an MCP tool call by routing to the appropriate MCP server.
    async fn handle_mcp_tool(
        &mut self,
        tool_name: &str,
        args_json: &str,
    ) -> (String, bool) {
        let mcp = match self.mcp.as_mut() {
            Some(m) => m,
            None => return ("No MCP servers connected".to_string(), false),
        };

        let server_name = match mcp.find_tool_server(tool_name) {
            Some(name) => name.to_string(),
            None => return (format!("No MCP server provides tool '{}'", tool_name), false),
        };

        let actual_tool_name = mcp.resolve_tool_name(tool_name).unwrap_or(tool_name.to_string());

        let args: serde_json::Value = serde_json::from_str(args_json)
            .unwrap_or(serde_json::Value::Object(Default::default()));

        match mcp.call_tool(&server_name, &actual_tool_name, args).await {
            Ok(result) => {
                let text = result
                    .content
                    .iter()
                    .filter_map(|c| c.text.as_ref())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n");
                (!result.is_error).then(|| (text.clone(), true))
                    .unwrap_or((text, false))
            }
            Err(e) => (format!("MCP error: {}", e), false),
        }
    }

    /// Clear the conversation history.
    pub fn reset(&mut self) {
        self.conversation.clear();
    }

    /// Get a reference to the conversation.
    pub fn conversation(&self) -> &Conversation {
        &self.conversation
    }

    /// Get cost tracking summary.
    pub fn cost_summary(&self) -> String {
        self.cost_tracker.summary()
    }
}
