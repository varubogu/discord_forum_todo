//! Discord-specific port traits.
//!
//! These traits represent the boundary between the `discord_task` adapter layer
//! and the actual Twilight HTTP client. Concrete implementations live in
//! `discord_task_infra` (Phase 6).

use core_lib::domain::{ChannelId, GuildId, MessageId, RoleId, Task, ThreadId};
use thiserror::Error;

// ── ForumService ──────────────────────────────────────────────────────────────

/// Errors returned by [`ForumService`].
#[derive(Debug, Error)]
pub enum ForumServiceError {
    /// The Discord HTTP request failed.
    #[error("discord http error: {0}")]
    Http(String),
}

/// Creates a new public thread inside a forum channel.
///
/// Used by [`ChannelCreateHandler`] when a watched channel name matches a
/// `Trigger` pattern: the handler creates a corresponding forum thread so that
/// the subsequent `ThreadCreate` gateway event registers a task.
///
/// [`ChannelCreateHandler`]: crate::events::ChannelCreateHandler
pub trait ForumService: Send + Sync {
    fn create_thread(
        &self,
        guild_id: GuildId,
        forum_channel_id: ChannelId,
        title: &str,
    ) -> impl std::future::Future<Output = Result<ThreadId, ForumServiceError>> + Send;
}

// ── ChannelInspector ──────────────────────────────────────────────────────────

/// Errors returned by [`ChannelInspector`].
#[derive(Debug, Error)]
pub enum ChannelInspectorError {
    /// The Discord HTTP request failed.
    #[error("discord http error: {0}")]
    Http(String),
}

/// Counts the guild's channels whose names match a given regex pattern.
///
/// Used by the `/count` command handler to tally channels per pattern.
pub trait ChannelInspector: Send + Sync {
    fn count_matching_channels(
        &self,
        guild_id: GuildId,
        pattern: &str,
    ) -> impl std::future::Future<Output = Result<u64, ChannelInspectorError>> + Send;
}

// ── ActiveMessageService ───────────────────────────────────────────────────────

/// Errors returned by [`ActiveMessageService`].
#[derive(Debug, Error)]
pub enum ActiveMessageServiceError {
    /// The Discord HTTP request failed.
    #[error("discord http error: {0}")]
    Http(String),
    /// A repository operation failed.
    #[error("repository error: {0}")]
    Repo(String),
}

/// Synchronises the active-tasks message in the bot's dedicated channel.
///
/// The service compares the provided `hash` against the stored hash; when they
/// differ it edits/re-posts the pages as one or more Discord messages.
/// Returns the list of current `message_ids` after synchronisation.
pub trait ActiveMessageService: Send + Sync {
    fn sync_active_message(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
        pages: Vec<String>,
        hash: Vec<u8>,
    ) -> impl std::future::Future<Output = Result<Vec<MessageId>, ActiveMessageServiceError>> + Send;
}

// ── TaskNotifier ───────────────────────────────────────────────────────────────

/// Errors returned by [`TaskNotifier`].
#[derive(Debug, Error)]
pub enum TaskNotifierError {
    /// The Discord HTTP request failed.
    #[error("discord http error: {0}")]
    Http(String),
}

/// Sends a stale-task notification in the guild's notification channel.
///
/// The message should mention the configured `role_id` and include a link to
/// the task thread so the responsible member can act on it.
pub trait TaskNotifier: Send + Sync {
    fn notify_stale(
        &self,
        channel_id: ChannelId,
        role_id: RoleId,
        task: &Task,
    ) -> impl std::future::Future<Output = Result<(), TaskNotifierError>> + Send;
}

// ── InteractionResponder ───────────────────────────────────────────────────────

/// Errors returned by [`InteractionResponder`].
#[derive(Debug, Error)]
pub enum InteractionResponderError {
    /// The Discord HTTP request failed.
    #[error("discord http error: {0}")]
    Http(String),
}

/// Sends a response to a Discord slash-command interaction.
///
/// Ephemeral responses are visible only to the invoking user; public responses
/// appear in the channel for everyone.
pub trait InteractionResponder: Send + Sync {
    /// Reply that only the invoking user can see.
    fn respond_ephemeral(
        &self,
        interaction_id: u64,
        interaction_token: &str,
        content: &str,
    ) -> impl std::future::Future<Output = Result<(), InteractionResponderError>> + Send;

    /// Reply visible to the whole channel.
    fn respond_public(
        &self,
        interaction_id: u64,
        interaction_token: &str,
        content: &str,
    ) -> impl std::future::Future<Output = Result<(), InteractionResponderError>> + Send;
}
