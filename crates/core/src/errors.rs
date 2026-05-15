//! Shared domain-level error types. Per-module error enums live alongside
//! the module that produces them; this file collects the cross-cutting ones.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("guild is not configured: {0}")]
    GuildNotConfigured(crate::domain::GuildId),

    #[error("invalid state transition")]
    InvalidStateTransition,

    #[error("not found")]
    NotFound,
}
