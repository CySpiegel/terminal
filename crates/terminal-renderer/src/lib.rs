//! GPU renderer for the terminal via wgpu.
//!
//! Provides glyph atlas management, cell batching, dirty tracking, and
//! shader-based rendering. Consumes `terminal-core` grid types but has
//! no terminal emulation logic of its own.

pub mod atlas;
pub mod batch;
pub mod color;
pub mod cursor;
pub mod damage;
pub mod font;
pub mod pipeline;
