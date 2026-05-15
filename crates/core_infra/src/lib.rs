//! `discord_forum_todo_core_infra` — adapter implementations for `core` ports.
//!
//! Currently exposes only a system clock; SeaORM repositories and the Fluent
//! translator land in Phase 4 and Phase 8 respectively.

pub mod clock;

pub use clock::SystemClock;
