//! Multi-agent system for parallel task execution.
//!
//! Provides an `AgentCoordinator` that can spawn sub-agents to work on
//! independent tasks in parallel, each running in its own tokio task.

use crate::agent::Agent;
use crate::client::LlmClient;
use crate::config::AiConfig;
use crate::permission::PermissionManager;
use crate::tools::ToolRegistry;
use tokio::sync::mpsc;

/// Unique identifier for a sub-agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(pub u64);

/// A sub-agent that wraps an `Agent` with its own conversation, tools, and role.
pub struct SubAgent {
    /// Unique identifier.
    pub id: AgentId,
    /// Role description (e.g., "code_reviewer", "test_writer").
    pub role: String,
    /// The underlying agent.
    pub agent: Agent,
    /// The task this sub-agent is working on.
    pub task: String,
}

/// Result produced by a sub-agent after completing its task.
#[derive(Debug, Clone)]
pub struct AgentResult {
    /// Which agent produced this result.
    pub agent_id: AgentId,
    /// The agent's role.
    pub role: String,
    /// The output text.
    pub output: String,
    /// Whether the agent completed successfully.
    pub success: bool,
}

/// Coordinates multiple sub-agents running in parallel.
pub struct AgentCoordinator {
    agents: Vec<SubAgent>,
    next_id: u64,
    results_tx: mpsc::Sender<AgentResult>,
    results_rx: mpsc::Receiver<AgentResult>,
    pending_count: usize,
}

impl AgentCoordinator {
    /// Create a new coordinator.
    pub fn new() -> Self {
        let (results_tx, results_rx) = mpsc::channel(64);
        Self {
            agents: Vec::new(),
            next_id: 0,
            results_tx,
            results_rx,
            pending_count: 0,
        }
    }

    /// Spawn a new sub-agent with the given role and task.
    ///
    /// The sub-agent gets a specialized system prompt based on its role.
    /// Returns the `AgentId` which can be used to identify results.
    pub fn spawn_agent(
        &mut self,
        role: &str,
        task: &str,
        config: AiConfig,
    ) -> AgentId {
        let id = AgentId(self.next_id);
        self.next_id += 1;

        let system_prompt = build_role_system_prompt(role);
        let client = LlmClient::new(&config);
        let tools = ToolRegistry::new();
        let permissions = PermissionManager::permissive();

        let agent = Agent::new(
            client,
            &system_prompt,
            tools,
            permissions,
            config.max_iterations,
        );

        let sub_agent = SubAgent {
            id,
            role: role.to_string(),
            agent,
            task: task.to_string(),
        };

        self.agents.push(sub_agent);
        id
    }

    /// Start all spawned agents running in parallel tokio tasks.
    ///
    /// Each agent processes its task independently and sends its result
    /// back via the internal channel.
    pub fn start_all(&mut self) {
        let agents: Vec<SubAgent> = self.agents.drain(..).collect();
        self.pending_count = agents.len();

        for mut sub_agent in agents {
            let tx = self.results_tx.clone();
            tokio::spawn(async move {
                let result = match sub_agent.agent.submit(&sub_agent.task).await {
                    Ok(mut rx) => {
                        let mut output = String::new();
                        while let Some(event) = rx.recv().await {
                            if let crate::agent::AgentEvent::Response(text) = event {
                                output = text;
                            }
                        }
                        AgentResult {
                            agent_id: sub_agent.id,
                            role: sub_agent.role.clone(),
                            output,
                            success: true,
                        }
                    }
                    Err(e) => AgentResult {
                        agent_id: sub_agent.id,
                        role: sub_agent.role.clone(),
                        output: format!("Agent error: {}", e),
                        success: false,
                    },
                };
                let _ = tx.send(result).await;
            });
        }
    }

    /// Wait for all pending agents to complete and collect their results.
    pub async fn wait_all(&mut self) -> Vec<AgentResult> {
        let mut results = Vec::with_capacity(self.pending_count);
        for _ in 0..self.pending_count {
            if let Some(result) = self.results_rx.recv().await {
                results.push(result);
            }
        }
        self.pending_count = 0;
        results
    }

    /// Wait for any one agent to complete and return its result.
    pub async fn wait_any(&mut self) -> AgentResult {
        self.pending_count = self.pending_count.saturating_sub(1);
        self.results_rx
            .recv()
            .await
            .expect("All agent senders dropped without sending a result")
    }
}

impl Default for AgentCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a specialized system prompt for a given agent role.
fn build_role_system_prompt(role: &str) -> String {
    match role {
        "code_reviewer" => {
            "You are a code review specialist. Analyze code for bugs, security \
             issues, performance problems, and style violations. Be thorough \
             but concise. Provide actionable feedback."
                .to_string()
        }
        "test_writer" => {
            "You are a test-writing specialist. Write comprehensive tests for \
             the given code. Cover edge cases, error conditions, and happy paths. \
             Use the project's existing test framework and conventions."
                .to_string()
        }
        "refactorer" => {
            "You are a refactoring specialist. Improve code structure, reduce \
             duplication, and enhance readability without changing behavior. \
             Explain your reasoning for each change."
                .to_string()
        }
        "documenter" => {
            "You are a documentation specialist. Write clear, accurate documentation \
             for code. Include doc comments, usage examples, and architecture notes."
                .to_string()
        }
        "debugger" => {
            "You are a debugging specialist. Systematically diagnose issues by \
             examining error messages, stack traces, and code flow. Identify root \
             causes and suggest minimal fixes."
                .to_string()
        }
        "searcher" => {
            "You are a codebase search specialist. Find relevant code, patterns, \
             and references across the project. Provide file paths and context \
             for each match."
                .to_string()
        }
        _ => {
            format!(
                "You are a specialist AI agent with the role: {}. \
                 Complete your assigned task thoroughly and report your findings.",
                role
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_equality() {
        let id1 = AgentId(0);
        let id2 = AgentId(0);
        let id3 = AgentId(1);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_coordinator_id_increments() {
        let mut coord = AgentCoordinator::new();
        let config = AiConfig::default();
        let id1 = coord.spawn_agent("test", "task1", config.clone());
        let id2 = coord.spawn_agent("test", "task2", config);
        assert_eq!(id1, AgentId(0));
        assert_eq!(id2, AgentId(1));
    }

    #[test]
    fn test_role_system_prompts() {
        let prompt = build_role_system_prompt("code_reviewer");
        assert!(prompt.contains("code review"));

        let prompt = build_role_system_prompt("custom_role");
        assert!(prompt.contains("custom_role"));
    }
}
