//! Port traits. Implementations live in `core_infra` (and `discord_task_infra`
//! for Discord-flavoured ports). Full method sets land in Phase 3+; this module
//! currently exposes the `Clock` port that the rest of the codebase already
//! references.

pub mod clock;

pub use clock::Clock;
