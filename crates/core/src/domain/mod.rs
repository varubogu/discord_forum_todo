//! Domain entities and value objects.
//!
//! Full implementations land in Phase 3; this module currently exposes the
//! newtype IDs that adapter crates already need to reference.

pub mod ids;

pub use ids::*;
