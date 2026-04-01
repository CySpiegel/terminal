//! GTD (Getting Things Done) task management system.
//!
//! Pure data and logic — no GUI dependencies. Provides SQLite-backed
//! persistence, pure-function operations, undo/redo, full-text search,
//! and scheduling with notification triggers.

pub mod model;
pub mod store;
pub mod operations;
pub mod undo;
pub mod search;
pub mod schedule;
pub mod export;
