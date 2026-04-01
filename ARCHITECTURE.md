# Architecture

GPU-rendered terminal emulator with built-in GTD, written in Rust using wgpu.

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `terminal-core` | Terminal emulation — VT parser, grid, PTY (zero GUI deps) |
| `terminal-renderer` | GPU renderer — wgpu pipeline, glyph atlas, cell batching |
| `terminal-gtd` | GTD task management — SQLite, pure operations, scheduling |
| `terminal-app` | Application — windowing, threading, input, GTD overlay |
| `terminal-platform` | Platform glue — font hints, desktop notifications |

## Threading Model (Ghostty-Style)

Each terminal surface gets 3 dedicated threads:

- **PTY Read Thread** — reads PTY output, parses VT sequences, updates grid
- **PTY Write Thread** — receives input from main thread, writes to PTY
- **Render Thread** — owns wgpu surface, renders on damage signal

Plus shared threads:
- **Main Thread** — winit event loop, input dispatch, tab/split management
- **GTD Thread** — SQLite operations, due date checks, notifications

## Rendering Pipeline

1. Check atomic damage flags
2. Read-lock grid, snapshot dirty rows
3. Resolve glyphs from atlas (rasterize on miss)
4. Build instance buffer
5. Submit 2-3 GPU draw calls (background, text, cursor)
6. Present (VSync)

## GTD Integration

Modal overlay toggled via hotkey. SQLite-backed with FTS5 search.
Vi-style keybindings. Pure-function operations for thread safety.

## Key Dependencies

wgpu, cosmic-text, etagere, vte, portable-pty, rusqlite, winit, parking_lot, crossbeam-channel
