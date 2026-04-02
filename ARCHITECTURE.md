# Architecture

GPU-rendered terminal emulator with built-in GTD and AI coding assistant, written in Rust using wgpu.

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `terminal-core` | Terminal emulation — VT parser, grid, PTY (zero GUI deps) |
| `terminal-renderer` | GPU renderer — wgpu pipeline, glyph atlas, cell batching |
| `terminal-gtd` | GTD task management — SQLite, pure operations, scheduling |
| `terminal-ai` | AI coding assistant — OpenAI-compatible LLM client, tool system, agent loop |
| `terminal-app` | Application — windowing, threading, input, GTD overlay, AI pane |
| `terminal-platform` | Platform glue — font hints, desktop notifications |

## Threading Model (Ghostty-Style)

Each terminal surface gets 3 dedicated threads:

- **PTY Read Thread** — reads PTY output, parses VT sequences, updates grid
- **PTY Write Thread** — receives input from main thread, writes to PTY
- **Render Thread** — owns wgpu surface, renders on damage signal

Plus shared threads:
- **Main Thread** — winit event loop, input dispatch, tab/split management
- **GTD Thread** — SQLite operations, due date checks, notifications
- **AI Thread** — tokio runtime for async LLM API calls, tool execution, streaming

## Rendering Pipeline

1. Check atomic damage flags
2. Read-lock grid, snapshot dirty rows
3. Resolve glyphs from atlas (rasterize on miss)
4. Build instance buffer
5. Submit 2-3 GPU draw calls (background, text, cursor)
6. Present (VSync)

## AI Assistant

Works with **any OpenAI-compatible API endpoint**:
- OpenAI, Anthropic (via proxy), LiteLLM, Ollama, vLLM, text-generation-inference
- Self-hosted models via any OpenAI-compatible proxy

**Architecture (inspired by Claude Code):**
- **LLM Client** — streaming chat completions via SSE
- **Tool System** — modular, permission-gated tools (file read/write/edit, bash, grep, glob)
- **Agent Loop** — iterative tool-call cycle (LLM → tool → result → LLM)
- **Slash Commands** — /commit, /review, /plan, /fix, /explain, /refactor, /test
- **Permission Manager** — controls which tools the AI can execute
- **Custom Tools** — register additional tools at runtime

**Configuration (config.toml):**
```toml
[ai]
base_url = "http://localhost:4000"  # LiteLLM proxy
model = "llama3.1:70b"
api_key = "sk-..."                  # optional for local models
max_tokens = 4096
temperature = 0.0
stream = true
```

## GTD Integration

Modal overlay toggled via hotkey. SQLite-backed with FTS5 search.
Vi-style keybindings. Pure-function operations for thread safety.

## Key Dependencies

wgpu, cosmic-text, etagere, vte, portable-pty, rusqlite, winit, parking_lot, crossbeam-channel, reqwest, tokio, eventsource-stream
