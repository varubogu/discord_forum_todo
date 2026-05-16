//! Gateway event handlers for `discord_forum_todo`.
//!
//! Each handler receives a [`twilight_gateway::Event`], ignores events it does
//! not care about, and delegates to core use-cases or Discord-specific ports.
//!
//! # Static dispatch
//!
//! [`AnyEventHandler`] is an enum that statically dispatches to one of the
//! concrete handler types. This avoids the vtable overhead of
//! `Box<dyn GatewayEventHandler>` while still letting the gateway layer work
//! with a single uniform type. Concrete type parameters are resolved in the
//! `discord_bot` binary crate (Phase 7).

use core_lib::domain::{CategoryId, ChannelPatternKind, GuildId, ThreadId};
use core_lib::ports::{ChannelPatternRepository, Clock, GuildSettingsRepository, TaskRepository};
use core_lib::usecases::CreateTaskFromThread;
use thiserror::Error;
use twilight_gateway::Event;

use crate::ports::ForumService;

// ── Error ─────────────────────────────────────────────────────────────────────

/// Errors returned by event handlers.
#[derive(Debug, Error)]
pub enum HandlerError {
    /// A task-repository or task use-case call failed.
    #[error("task error: {0}")]
    Task(String),
    /// A [`ForumService`] call failed.
    #[error("forum error: {0}")]
    Forum(String),
    /// A pattern-repository or guild-settings call failed.
    #[error("pattern error: {0}")]
    Pattern(String),
}

// ── GatewayEventHandler trait ────────────────────────────────────────────────

/// Uniform interface for all gateway event handlers.
///
/// Implementors receive every [`Event`] emitted by the Twilight gateway shard
/// and silently ignore events they do not handle (`Ok(())` fast-path).
pub trait GatewayEventHandler: Send + Sync {
    fn handle(
        &self,
        event: &Event,
    ) -> impl std::future::Future<Output = Result<(), HandlerError>> + Send;
}

// ── ThreadCreateHandler ───────────────────────────────────────────────────────

/// Handles `ThreadCreate` events: registers the new forum thread as a task.
///
/// The operation is idempotent — if a task already exists for the thread ID,
/// the use-case returns it unchanged.
pub struct ThreadCreateHandler<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    /// Inner use-case. Public so adapter integration tests can inspect the repo.
    pub use_case: CreateTaskFromThread<R, C>,
}

impl<R, C> ThreadCreateHandler<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    /// Construct the handler.
    pub fn new(repo: R, clock: C) -> Self {
        Self {
            use_case: CreateTaskFromThread::new(repo, clock),
        }
    }
}

impl<R, C> GatewayEventHandler for ThreadCreateHandler<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    async fn handle(&self, event: &Event) -> Result<(), HandlerError> {
        let Event::ThreadCreate(tc) = event else {
            return Ok(());
        };
        let Some(gid) = tc.guild_id else {
            return Ok(());
        };
        let guild_id = GuildId(gid.get());
        let thread_id = ThreadId(tc.id.get());
        let title = tc.name.clone().unwrap_or_default();

        self.use_case
            .execute(guild_id, thread_id, title)
            .await
            .map_err(|e| HandlerError::Task(e.to_string()))?;

        Ok(())
    }
}

// ── ThreadDeleteHandler ───────────────────────────────────────────────────────

/// Handles `ThreadDelete` events: soft-deletes the corresponding task.
///
/// If no task is mapped to the thread (e.g. the thread was never in the
/// forum channel), the handler returns `Ok(())` silently.
pub struct ThreadDeleteHandler<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    /// Task repository. Public so adapter integration tests can inspect state.
    pub repo: R,
    pub clock: C,
}

impl<R, C> ThreadDeleteHandler<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    /// Construct the handler.
    pub fn new(repo: R, clock: C) -> Self {
        Self { repo, clock }
    }
}

impl<R, C> GatewayEventHandler for ThreadDeleteHandler<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    async fn handle(&self, event: &Event) -> Result<(), HandlerError> {
        let Event::ThreadDelete(td) = event else {
            return Ok(());
        };
        let thread_id = ThreadId(td.id.get());

        let Some(task) = self
            .repo
            .find_by_thread(thread_id)
            .await
            .map_err(|e| HandlerError::Task(e.to_string()))?
        else {
            return Ok(()); // Thread was never tracked, ignore.
        };

        let now = self.clock.now();
        self.repo
            .soft_delete(task.id, now)
            .await
            .map_err(|e| HandlerError::Task(e.to_string()))?;

        Ok(())
    }
}

// ── ChannelCreateHandler ──────────────────────────────────────────────────────

/// Handles `ChannelCreate` events.
///
/// When a new channel appears inside the guild's configured watch category and
/// its name matches a `Trigger` channel pattern, the handler creates a forum
/// thread via [`ForumService`]. The subsequent `ThreadCreate` event from
/// Discord then registers the new task automatically.
pub struct ChannelCreateHandler<GsR, PatternR, ForumSvc>
where
    GsR: GuildSettingsRepository,
    PatternR: ChannelPatternRepository,
    ForumSvc: ForumService,
{
    gs_repo: GsR,
    pattern_repo: PatternR,
    forum_service: ForumSvc,
}

impl<GsR, PatternR, ForumSvc> ChannelCreateHandler<GsR, PatternR, ForumSvc>
where
    GsR: GuildSettingsRepository,
    PatternR: ChannelPatternRepository,
    ForumSvc: ForumService,
{
    /// Construct the handler.
    pub fn new(gs_repo: GsR, pattern_repo: PatternR, forum_service: ForumSvc) -> Self {
        Self {
            gs_repo,
            pattern_repo,
            forum_service,
        }
    }
}

impl<GsR, PatternR, ForumSvc> GatewayEventHandler for ChannelCreateHandler<GsR, PatternR, ForumSvc>
where
    GsR: GuildSettingsRepository,
    PatternR: ChannelPatternRepository,
    ForumSvc: ForumService,
{
    async fn handle(&self, event: &Event) -> Result<(), HandlerError> {
        let Event::ChannelCreate(cc) = event else {
            return Ok(());
        };
        let Some(gid) = cc.guild_id else {
            return Ok(());
        };
        let guild_id = GuildId(gid.get());

        // Load guild settings to discover the watch category and forum channel.
        let settings = self
            .gs_repo
            .find(guild_id)
            .await
            .map_err(|e| HandlerError::Pattern(e.to_string()))?;
        let Some(settings) = settings else {
            return Ok(());
        };

        // Both watch_category_id and forum_channel_id must be configured.
        let Some(watch_category_id) = settings.watch_category_id else {
            return Ok(());
        };
        let Some(forum_channel_id) = settings.forum_channel_id else {
            return Ok(());
        };

        // The created channel must be inside the watched category.
        let Some(parent_id) = cc.parent_id else {
            return Ok(());
        };
        if parent_id.get() != CategoryId::get(watch_category_id) {
            return Ok(());
        }

        let channel_name = cc.name.as_deref().unwrap_or("");

        // Load trigger patterns and check if any match.
        let patterns = self
            .pattern_repo
            .list(guild_id)
            .await
            .map_err(|e| HandlerError::Pattern(e.to_string()))?;

        for p in &patterns {
            if p.kind != ChannelPatternKind::Trigger {
                continue;
            }
            if regex_matches(&p.pattern, channel_name) {
                self.forum_service
                    .create_thread(guild_id, forum_channel_id, channel_name)
                    .await
                    .map_err(|e| HandlerError::Forum(e.to_string()))?;
                // Create at most one thread per channel-create event.
                break;
            }
        }

        Ok(())
    }
}

// ── AnyEventHandler ───────────────────────────────────────────────────────────

/// A statically-dispatched union of all event handler types.
///
/// The enum achieves the same zero-cost dispatch as `enum_dispatch` would
/// provide — each arm of the `handle` match is monomorphised with the concrete
/// handler type. Concrete type parameters are filled in the `discord_bot`
/// binary (Phase 7).
///
/// # Type parameters
///
/// | Parameter | Bound | Used by |
/// |-----------|-------|---------|
/// | `TaskR` | `TaskRepository` | Thread{Create,Delete}Handler |
/// | `C` | `Clock` | Thread{Create,Delete}Handler |
/// | `GsR` | `GuildSettingsRepository` | ChannelCreateHandler |
/// | `PatternR` | `ChannelPatternRepository` | ChannelCreateHandler |
/// | `ForumSvc` | `ForumService` | ChannelCreateHandler |
pub enum AnyEventHandler<TaskR, C, GsR, PatternR, ForumSvc>
where
    TaskR: TaskRepository,
    C: Clock,
    GsR: GuildSettingsRepository,
    PatternR: ChannelPatternRepository,
    ForumSvc: ForumService,
{
    /// Handles `ThreadCreate` — registers a new task.
    ThreadCreate(ThreadCreateHandler<TaskR, C>),
    /// Handles `ThreadDelete` — soft-deletes the task.
    ThreadDelete(ThreadDeleteHandler<TaskR, C>),
    /// Handles `ChannelCreate` — creates a forum thread for trigger patterns.
    ChannelCreate(ChannelCreateHandler<GsR, PatternR, ForumSvc>),
}

impl<TaskR, C, GsR, PatternR, ForumSvc> GatewayEventHandler
    for AnyEventHandler<TaskR, C, GsR, PatternR, ForumSvc>
where
    TaskR: TaskRepository,
    C: Clock,
    GsR: GuildSettingsRepository,
    PatternR: ChannelPatternRepository,
    ForumSvc: ForumService,
{
    async fn handle(&self, event: &Event) -> Result<(), HandlerError> {
        match self {
            Self::ThreadCreate(h) => h.handle(event).await,
            Self::ThreadDelete(h) => h.handle(event).await,
            Self::ChannelCreate(h) => h.handle(event).await,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns `true` if `name` matches the compiled regex `pattern`.
/// Invalid patterns are treated as non-matching.
fn regex_matches(pattern: &str, name: &str) -> bool {
    regex::Regex::new(pattern)
        .map(|re| re.is_match(name))
        .unwrap_or(false)
}
