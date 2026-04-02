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
            name: "compact".to_string(),
            description: "Compress conversation history to save context".to_string(),
            handler: CommandHandler::Prompt(|_args| {
                "Summarize the entire conversation so far into a compact form. \
                 Preserve all key decisions, code changes, file paths, and action items. \
                 Present the summary so it can replace the older messages without losing \
                 important context."
                    .to_string()
            }),
        });

        registry.register(SlashCommand {
            name: "init".to_string(),
            description: "Analyze codebase and set up project conventions".to_string(),
            handler: CommandHandler::Prompt(|_args| {
                "Analyze the project structure in the current directory. Identify the \
                 programming language(s), framework(s), build system, and test framework. \
                 Suggest coding conventions, naming patterns, and best practices for this \
                 project. Summarize the architecture at a high level."
                    .to_string()
            }),
        });

        registry.register(SlashCommand {
            name: "diff".to_string(),
            description: "Show and explain git diff".to_string(),
            handler: CommandHandler::Prompt(|args| {
                if args.is_empty() {
                    "Run `git diff` and `git diff --staged` to see all current changes. \
                     Explain what the changes do, highlight anything risky, and suggest \
                     improvements if appropriate."
                        .to_string()
                } else {
                    format!(
                        "Run `git diff {}` and explain the changes. Highlight anything \
                         risky and suggest improvements if appropriate.",
                        args
                    )
                }
            }),
        });

        registry.register(SlashCommand {
            name: "doctor".to_string(),
            description: "Diagnose environment issues".to_string(),
            handler: CommandHandler::Direct(|_args| {
                let mut info = Vec::new();
                info.push(format!("OS: {}", std::env::consts::OS));
                info.push(format!("Arch: {}", std::env::consts::ARCH));
                if let Ok(path) = std::env::var("PATH") {
                    let tool_count = path.split(':').count();
                    info.push(format!("PATH entries: {}", tool_count));
                }
                if let Ok(shell) = std::env::var("SHELL") {
                    info.push(format!("Shell: {}", shell));
                }
                if let Ok(home) = std::env::var("HOME") {
                    info.push(format!("Home: {}", home));
                }
                for tool in &["git", "cargo", "rustc", "node", "python3", "docker"] {
                    let status = std::process::Command::new("which")
                        .arg(tool)
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);
                    info.push(format!(
                        "{}: {}",
                        tool,
                        if status { "found" } else { "not found" }
                    ));
                }
                info.join("\n")
            }),
        });

        registry.register(SlashCommand {
            name: "batch".to_string(),
            description: "Run a task across multiple files".to_string(),
            handler: CommandHandler::Prompt(|args| {
                format!(
                    "Run the following task across multiple files: {}. \
                     Use glob patterns to find the matching files, then apply the \
                     instruction to each file. Report what was changed.",
                    args
                )
            }),
        });

        registry.register(SlashCommand {
            name: "simplify".to_string(),
            description: "Review code for reuse and efficiency".to_string(),
            handler: CommandHandler::Prompt(|args| {
                if args.is_empty() {
                    "Review the recent changes (git diff) for simplification opportunities. \
                     Look for duplicated logic, overly complex code, unused imports, and \
                     chances to reuse existing utilities. Suggest concrete improvements."
                        .to_string()
                } else {
                    format!(
                        "Review the following code for simplification opportunities: {}. \
                         Look for duplicated logic, overly complex code, and chances to \
                         reuse existing utilities.",
                        args
                    )
                }
            }),
        });

        registry.register(SlashCommand {
            name: "memory".to_string(),
            description: "Show or edit persistent memory".to_string(),
            handler: CommandHandler::Direct(|args| {
                if args.is_empty() {
                    "Memory system placeholder. Use /memory <key>=<value> to set, \
                     /memory <key> to get, or /memory --list to list all entries."
                        .to_string()
                } else {
                    format!("Memory operation: {}", args)
                }
            }),
        });

        registry.register(SlashCommand {
            name: "context".to_string(),
            description: "Show context window usage".to_string(),
            handler: CommandHandler::Direct(|_args| {
                "Context window usage information is not available in this handler. \
                 The conversation manager tracks context usage."
                    .to_string()
            }),
        });

        registry.register(SlashCommand {
            name: "status".to_string(),
            description: "Show AI assistant status".to_string(),
            handler: CommandHandler::Direct(|_args| {
                "AI assistant is running. Use /model to see the current model, \
                 /config to see configuration, and /context for context usage."
                    .to_string()
            }),
        });

        registry.register(SlashCommand {
            name: "search".to_string(),
            description: "Search the codebase".to_string(),
            handler: CommandHandler::Prompt(|args| {
                format!(
                    "Search the codebase for: {}. \
                     Use grep and glob tools to find all relevant occurrences. \
                     Report the matching files and the relevant code snippets.",
                    args
                )
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
