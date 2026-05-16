//! Slash-command handlers for `discord_forum_todo`.
//!
//! Each handler owns the use-case and the [`InteractionResponder`] port needed
//! to reply to Discord. Inputs are plain Rust structs — parsing raw
//! [`twilight_model::application::interaction::Interaction`] data into these
//! structs happens in the gateway/adapter layer (Phase 7).
//!
//! [`InteractionResponder`]: crate::ports::InteractionResponder

use core_lib::domain::{GuildId, TaskId, TaskState, UserId};
use core_lib::ports::{
    ChannelPatternRepository, Clock, GuildSettingsRepository, GuildSettingsUpdate, TaskRepository,
};
use core_lib::usecases::{RegisterGuildSettings, TransitionTaskState};
use thiserror::Error;

use crate::ports::{ChannelInspector, InteractionResponder};

// ── Shared error ──────────────────────────────────────────────────────────────

/// Top-level error type returned by all command handlers.
#[derive(Debug, Error)]
pub enum CommandError {
    /// A core use-case or repository call failed.
    #[error("use-case error: {0}")]
    UseCase(String),
    /// Sending the interaction response failed.
    #[error("responder error: {0}")]
    Responder(String),
    /// The channel-inspector call failed (`/count` only).
    #[error("inspector error: {0}")]
    Inspector(String),
}

// ── /config set ───────────────────────────────────────────────────────────────

/// Input for the `/config set <field> <value>` command.
#[derive(Debug)]
pub struct ConfigSetInput {
    /// Guild in which the command was invoked.
    pub guild_id: GuildId,
    /// Discord interaction ID (from the interaction payload).
    pub interaction_id: u64,
    /// Discord interaction token (from the interaction payload).
    pub interaction_token: String,
    /// The single field mutation to apply.
    pub update: GuildSettingsUpdate,
    /// Discord user who invoked the command (for audit trail).
    pub actor: Option<UserId>,
}

/// Handler for `/config set <field> <value>`.
///
/// Calls [`RegisterGuildSettings`] and replies ephemerally on success.
pub struct ConfigSetHandler<R, C, Resp>
where
    R: GuildSettingsRepository,
    C: Clock,
    Resp: InteractionResponder,
{
    /// Inner use-case (accessible for integration tests via `pub`).
    pub use_case: RegisterGuildSettings<R, C>,
    /// Responder port (accessible for integration tests via `pub`).
    pub responder: Resp,
}

impl<R, C, Resp> ConfigSetHandler<R, C, Resp>
where
    R: GuildSettingsRepository,
    C: Clock,
    Resp: InteractionResponder,
{
    /// Construct the handler, taking ownership of all dependencies.
    pub fn new(repo: R, clock: C, responder: Resp) -> Self {
        Self {
            use_case: RegisterGuildSettings::new(repo, clock),
            responder,
        }
    }

    /// Execute the handler for the given input.
    pub async fn handle(&self, input: ConfigSetInput) -> Result<(), CommandError> {
        self.use_case
            .execute(input.guild_id, input.update, input.actor)
            .await
            .map_err(|e| CommandError::UseCase(e.to_string()))?;

        self.responder
            .respond_ephemeral(
                input.interaction_id,
                &input.interaction_token,
                "✅ Setting updated.",
            )
            .await
            .map_err(|e| CommandError::Responder(e.to_string()))?;

        Ok(())
    }
}

// ── /task <state> ─────────────────────────────────────────────────────────────

/// Input for the `/task done|cancel|start` commands.
#[derive(Debug)]
pub struct TaskStateInput {
    /// Internal task primary key.
    pub task_id: TaskId,
    /// The state to transition the task into.
    pub next_state: TaskState,
    /// Discord interaction ID.
    pub interaction_id: u64,
    /// Discord interaction token.
    pub interaction_token: String,
}

/// Handler for state-transition slash commands (`/task done`, `/task cancel`, …).
///
/// Calls [`TransitionTaskState`] and replies publicly with the new state.
pub struct TaskStateHandler<R, C, Resp>
where
    R: TaskRepository,
    C: Clock,
    Resp: InteractionResponder,
{
    /// Inner use-case.
    pub use_case: TransitionTaskState<R, C>,
    /// Responder port.
    pub responder: Resp,
}

impl<R, C, Resp> TaskStateHandler<R, C, Resp>
where
    R: TaskRepository,
    C: Clock,
    Resp: InteractionResponder,
{
    /// Construct the handler.
    pub fn new(repo: R, clock: C, responder: Resp) -> Self {
        Self {
            use_case: TransitionTaskState::new(repo, clock),
            responder,
        }
    }

    /// Execute the handler for the given input.
    pub async fn handle(&self, input: TaskStateInput) -> Result<(), CommandError> {
        let task = self
            .use_case
            .execute(input.task_id, input.next_state)
            .await
            .map_err(|e| CommandError::UseCase(e.to_string()))?;

        let msg = format!("✅ **{}** → `{}`", task.title, task.state.as_str());
        self.responder
            .respond_public(input.interaction_id, &input.interaction_token, &msg)
            .await
            .map_err(|e| CommandError::Responder(e.to_string()))?;

        Ok(())
    }
}

// ── /count ────────────────────────────────────────────────────────────────────

/// Input for the `/count [pattern_name]` command.
#[derive(Debug)]
pub struct CountInput {
    /// Guild in which the command was invoked.
    pub guild_id: GuildId,
    /// When `Some`, only the named pattern is counted; `None` counts all patterns.
    pub pattern_name: Option<String>,
    /// Discord interaction ID.
    pub interaction_id: u64,
    /// Discord interaction token.
    pub interaction_token: String,
}

/// Handler for the `/count` command.
///
/// Lists the guild's `Counter`-kind channel patterns, uses [`ChannelInspector`]
/// to count matching channels for each, and replies publicly.
pub struct CountHandler<PatternR, Insp, Resp>
where
    PatternR: ChannelPatternRepository,
    Insp: ChannelInspector,
    Resp: InteractionResponder,
{
    /// Pattern repository.
    pub pattern_repo: PatternR,
    /// Channel inspector port.
    pub inspector: Insp,
    /// Responder port.
    pub responder: Resp,
}

impl<PatternR, Insp, Resp> CountHandler<PatternR, Insp, Resp>
where
    PatternR: ChannelPatternRepository,
    Insp: ChannelInspector,
    Resp: InteractionResponder,
{
    /// Construct the handler.
    pub fn new(pattern_repo: PatternR, inspector: Insp, responder: Resp) -> Self {
        Self {
            pattern_repo,
            inspector,
            responder,
        }
    }

    /// Execute the handler for the given input.
    pub async fn handle(&self, input: CountInput) -> Result<(), CommandError> {
        use core_lib::domain::ChannelPatternKind;

        let patterns = self
            .pattern_repo
            .list(input.guild_id)
            .await
            .map_err(|e| CommandError::UseCase(e.to_string()))?;

        let mut lines: Vec<String> = Vec::new();

        for p in &patterns {
            // Only tally Counter-kind patterns.
            if p.kind != ChannelPatternKind::Counter {
                continue;
            }
            // Filter by name if the user specified one.
            if let Some(ref name) = input.pattern_name {
                if &p.name != name {
                    continue;
                }
            }

            let count = self
                .inspector
                .count_matching_channels(input.guild_id, &p.pattern)
                .await
                .map_err(|e| CommandError::Inspector(e.to_string()))?;

            lines.push(format!("**{}**: {}", p.name, count));
        }

        let msg = if lines.is_empty() {
            "No matching patterns found.".to_string()
        } else {
            lines.join("\n")
        };

        self.responder
            .respond_public(input.interaction_id, &input.interaction_token, &msg)
            .await
            .map_err(|e| CommandError::Responder(e.to_string()))?;

        Ok(())
    }
}
