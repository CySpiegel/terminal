//! Slash commands for the AI assistant.
//!
//! Commands are user-invokable shortcuts prefixed with `/`.
//! Inspired by Claude Code's 55+ command system.

use std::collections::HashMap;

/// A registered slash command.
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub handler: CommandHandler,
}

/// Command handler — returns a prompt to send to the LLM or executes directly.
pub enum CommandHandler {
    /// Rewrite user input into a specialized prompt for the LLM.
    Prompt(fn(&str) -> String),
    /// Execute directly without LLM involvement.
    Direct(fn(&str) -> String),
}

/// Registry of available slash commands.
pub struct CommandRegistry {
    commands: HashMap<String, SlashCommand>,
}

impl CommandRegistry {
    pub fn with_builtins() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };

        registry.register(SlashCommand {
            name: "commit".to_string(),
            description: "Create a git commit with AI-generated message".to_string(),
            handler: CommandHandler::Prompt(|_args| {
                "Look at the current git diff (staged and unstaged) and recent commit history. \
                 Create a concise, descriptive commit message. Stage the relevant files and \
                 commit. Do not push."
                    .to_string()
            }),
        });

        registry.register(SlashCommand {
            name: "review".to_string(),
            description: "Review code changes".to_string(),
            handler: CommandHandler::Prompt(|args| {
                if args.is_empty() {
                    "Review the current git diff for bugs, security issues, and improvements. \
                     Be concise and actionable."
                        .to_string()
                } else {
                    format!(
                        "Review the following code or file for bugs, security issues, \
                         and improvements: {}",
                        args
                    )
                }
            }),
        });

        registry.register(SlashCommand {
            name: "plan".to_string(),
            description: "Design an implementation plan without making changes".to_string(),
            handler: CommandHandler::Prompt(|args| {
                format!(
                    "Design an implementation plan for: {}. \
                     Identify the files to change, the approach, and any risks. \
                     Do NOT make any changes — just plan.",
                    args
                )
            }),
        });

        registry.register(SlashCommand {
            name: "explain".to_string(),
            description: "Explain how code works".to_string(),
            handler: CommandHandler::Prompt(|args| {
                format!(
                    "Explain how this code/system works: {}. \
                     Be thorough but concise. Include the key design decisions.",
                    args
                )
            }),
        });

        registry.register(SlashCommand {
            name: "fix".to_string(),
            description: "Fix a bug or error".to_string(),
            handler: CommandHandler::Prompt(|args| {
                format!(
                    "Fix this bug/error: {}. \
                     Diagnose the root cause, then make the minimal fix needed.",
                    args
                )
            }),
        });

        registry.register(SlashCommand {
            name: "refactor".to_string(),
            description: "Refactor code".to_string(),
            handler: CommandHandler::Prompt(|args| {
                format!(
                    "Refactor: {}. \
                     Improve the code structure without changing behavior.",
                    args
                )
            }),
        });

        registry.register(SlashCommand {
            name: "test".to_string(),
            description: "Write or run tests".to_string(),
            handler: CommandHandler::Prompt(|args| {
                if args.is_empty() {
                    "Run the test suite and report results.".to_string()
                } else {
                    format!("Write tests for: {}", args)
                }
            }),
        });

        registry.register(SlashCommand {
            name: "help".to_string(),
            description: "Show available commands".to_string(),
            handler: CommandHandler::Direct(|_args| {
                "Use /help to see this list. Available commands are shown below.".to_string()
            }),
        });

        registry.register(SlashCommand {
            name: "clear".to_string(),
            description: "Clear conversation history".to_string(),
            handler: CommandHandler::Direct(|_args| {
                "Conversation cleared.".to_string()
            }),
        });

        registry.register(SlashCommand {
            name: "model".to_string(),
            description: "Show or change the current model".to_string(),
            handler: CommandHandler::Direct(|args| {
                if args.is_empty() {
                    "Current model shown above. Use /model <name> to change.".to_string()
                } else {
                    format!("Model changed to: {}", args)
                }
            }),
        });

        registry.register(SlashCommand {
            name: "config".to_string(),
            description: "Show or edit AI configuration".to_string(),
            handler: CommandHandler::Direct(|_args| {
                "Configuration shown above. Edit ~/.config/terminal/config.toml to change."
                    .to_string()
            }),
        });

        registry
    }

    fn register(&mut self, cmd: SlashCommand) {
        self.commands.insert(cmd.name.clone(), cmd);
    }

    /// Parse and execute a slash command. Returns the prompt to send to the LLM,
    /// or None if the command was handled directly.
    pub fn execute(&self, input: &str) -> Option<CommandResult> {
        let input = input.trim();
        if !input.starts_with('/') {
            return None;
        }

        let (cmd_name, args) = match input[1..].split_once(char::is_whitespace) {
            Some((name, args)) => (name, args.trim()),
            None => (&input[1..], ""),
        };

        let cmd = self.commands.get(cmd_name)?;

        match &cmd.handler {
            CommandHandler::Prompt(f) => Some(CommandResult::Prompt(f(args))),
            CommandHandler::Direct(f) => Some(CommandResult::Direct(f(args))),
        }
    }

    /// List all available commands.
    pub fn list(&self) -> Vec<(&str, &str)> {
        let mut cmds: Vec<_> = self
            .commands
            .values()
            .map(|c| (c.name.as_str(), c.description.as_str()))
            .collect();
        cmds.sort_by_key(|(name, _)| *name);
        cmds
    }
}

/// Result of executing a slash command.
pub enum CommandResult {
    /// Send this prompt to the LLM.
    Prompt(String),
    /// Command was handled directly; display this message.
    Direct(String),
}
