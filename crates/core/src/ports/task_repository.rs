//! `TaskRepository` — persistence of tasks plus the queries needed by the
//! stale-detection scheduler.

use crate::domain::{GuildId, NewTask, Task, TaskId, TaskState, ThreadId};
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskRepoError {
    #[error("task not found")]
    NotFound,
    #[error("thread id already mapped to a task")]
    ThreadAlreadyMapped,
    #[error("underlying store: {0}")]
    Backend(String),
}

/// Snapshot of a stale task plus the guild bits needed to notify about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleTaskRow {
    pub task: Task,
    pub user_role_id: crate::domain::RoleId,
    pub notify_channel_id: crate::domain::ChannelId,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct StaleQuery<'a> {
    pub now: DateTime<Utc>,
    pub batch_size: u32,
    pub guild_filter: Option<&'a [GuildId]>,
}

pub trait TaskRepository: Send + Sync {
    fn insert(
        &self,
        task: NewTask,
    ) -> impl std::future::Future<Output = Result<Task, TaskRepoError>> + Send;

    fn find_by_thread(
        &self,
        thread_id: ThreadId,
    ) -> impl std::future::Future<Output = Result<Option<Task>, TaskRepoError>> + Send;

    fn fetch_active(
        &self,
        guild_id: GuildId,
    ) -> impl std::future::Future<Output = Result<Vec<Task>, TaskRepoError>> + Send;

    fn transition_state(
        &self,
        id: TaskId,
        next: TaskState,
        at: DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<Task, TaskRepoError>> + Send;

    fn soft_delete(
        &self,
        id: TaskId,
        at: DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<(), TaskRepoError>> + Send;

    /// Returns stale tasks ready for notification along with the guild context
    /// needed to actually send the mention. Implementations should apply the
    /// cooldown predicate and a `FOR UPDATE SKIP LOCKED` (or equivalent) lock
    /// so multiple scheduler instances can coexist safely.
    fn fetch_stale_for_notify(
        &self,
        query: StaleQuery<'_>,
    ) -> impl std::future::Future<Output = Result<Vec<StaleTaskRow>, TaskRepoError>> + Send;

    fn mark_notified(
        &self,
        ids: &[TaskId],
        at: DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<(), TaskRepoError>> + Send;
}
