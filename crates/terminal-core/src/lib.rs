//! Terminal emulation library.
//!
//! Provides VT100/ANSI terminal parsing, grid state management, and PTY
//! abstraction with zero GUI dependencies. Designed to be embedded in any
//! frontend.

pub mod event;
pub mod grid;
pub mod pty;
pub mod vt;

mod selection;
mod search;
mod term;

pub use term::Terminal;
pub use selection::Selection;
pub use search::SearchMatch;
