//! `discord_forum_todo_core` — pure domain layer.
//!
//! No Discord, no database, no filesystem. Side effects reach this crate only
//! through ports (traits) implemented by adapter crates.

pub mod domain;
pub mod errors;
pub mod i18n;
pub mod ports;
pub mod usecases;

pub use domain::*;
