//! AI coding assistant for the terminal.
//!
//! Provides an LLM-powered coding assistant that works with **any
//! OpenAI-compatible API endpoint** — Anthropic, OpenAI, LiteLLM,
//! Ollama, vLLM, or any self-hosted model.
//!
//! Architecture inspired by Claude Code's modular tool system:
//! - **LLM Client** — streams completions from any OpenAI-compatible endpoint
//! - **Tool System** — modular, permission-gated tools (file read/write/edit,
//!   grep, bash, web fetch, etc.)
//! - **Agent Loop** — iterative tool-call loop (LLM calls tool → execute → return result)
//! - **Commands** — slash commands for common workflows (/commit, /review, /plan, etc.)
//! - **Multi-Agent** — spawn sub-agents for parallel tasks

pub mod client;
pub mod config;
pub mod tools;
pub mod commands;
pub mod agent;
pub mod conversation;
pub mod permission;
pub mod streaming;
