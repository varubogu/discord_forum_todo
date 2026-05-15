//! Task entity, value objects, and state-transition rules.

use crate::domain::ids::{GuildId, ThreadId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Database-assigned task primary key. Different from the Discord thread id;
/// kept distinct so the application owns its own identity space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaskId(pub i64);

impl TaskId {
    #[inline]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }
    #[inline]
    pub const fn get(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// All states a task can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    NotStarted,
    InProgress,
    Done,
    Cancelled,
}

impl TaskState {
    pub const ALL: [TaskState; 4] = [
        TaskState::NotStarted,
        TaskState::InProgress,
        TaskState::Done,
        TaskState::Cancelled,
    ];

    /// String form persisted in the database. Must stay aligned with the
    /// CHECK constraint on `tasks.state`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            TaskState::NotStarted => "not_started",
            TaskState::InProgress => "in_progress",
            TaskState::Done => "done",
            TaskState::Cancelled => "cancelled",
        }
    }

    /// Parse the canonical persisted form. Returns `None` for unknown input.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "not_started" => Some(TaskState::NotStarted),
            "in_progress" => Some(TaskState::InProgress),
            "done" => Some(TaskState::Done),
            "cancelled" => Some(TaskState::Cancelled),
            _ => None,
        }
    }

    /// Whether this state counts as "open" — i.e. it can become stale and must
    /// show up in the active-tasks summary message.
    #[must_use]
    pub fn is_open(self) -> bool {
        matches!(self, TaskState::NotStarted | TaskState::InProgress)
    }

    /// Whether `self -> next` is a permitted transition.
    #[must_use]
    pub fn can_transition_to(self, next: TaskState) -> bool {
        if self == next {
            return false;
        }
        // Any open ↔ open transition is allowed.
        // From open you can move to done or cancelled.
        // From done/cancelled you can reopen back to not_started or in_progress
        // (users sometimes change their mind; the audit log captures who did it).
        true
    }
}

/// A persisted task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: TaskId,
    pub guild_id: GuildId,
    pub project_id: Option<i64>,
    pub thread_id: ThreadId,
    pub title: String,
    pub state: TaskState,
    pub created_at: DateTime<Utc>,
    pub last_state_changed_at: DateTime<Utc>,
    pub last_notified_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// A task that has not been persisted yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTask {
    pub guild_id: GuildId,
    pub project_id: Option<i64>,
    pub thread_id: ThreadId,
    pub title: String,
    pub state: TaskState,
    pub created_at: DateTime<Utc>,
}

impl NewTask {
    /// Construct a freshly created not-started task at `now`.
    #[must_use]
    pub fn fresh(
        guild_id: GuildId,
        thread_id: ThreadId,
        title: impl Into<String>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            guild_id,
            project_id: None,
            thread_id,
            title: title.into(),
            state: TaskState::NotStarted,
            created_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trip() {
        for s in TaskState::ALL {
            assert_eq!(TaskState::parse(s.as_str()), Some(s));
        }
    }

    #[test]
    fn state_is_open() {
        assert!(TaskState::NotStarted.is_open());
        assert!(TaskState::InProgress.is_open());
        assert!(!TaskState::Done.is_open());
        assert!(!TaskState::Cancelled.is_open());
    }

    #[test]
    fn transition_to_self_is_rejected() {
        for s in TaskState::ALL {
            assert!(!s.can_transition_to(s), "{s:?} -> {s:?} should be rejected");
        }
    }

    #[test]
    fn transitions_open_to_terminal_allowed() {
        assert!(TaskState::NotStarted.can_transition_to(TaskState::Done));
        assert!(TaskState::InProgress.can_transition_to(TaskState::Cancelled));
    }
}
