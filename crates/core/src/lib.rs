//! `discord_forum_todo_core` — pure domain layer.
//!
//! No Discord, no database, no filesystem. Side effects reach this crate only
//! through ports (traits) implemented by adapter crates.

pub mod domain;
pub mod errors;
pub mod i18n;
pub mod ports;
pub mod usecases;

/// In-memory fakes for the ports declared in `core::ports`. Kept public so
/// integration tests and downstream crates can compose them; release builds
/// drop unused code via dead-code elimination.
pub mod test_support;

pub use domain::*;
