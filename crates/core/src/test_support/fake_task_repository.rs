use std::sync::Mutex;

use chrono::{DateTime, Utc};

use crate::domain::{GuildId, NewTask, Task, TaskId, TaskState, ThreadId};
use crate::ports::{StaleQuery, StaleTaskRow, TaskRepoError, TaskRepository};

/// In-memory `TaskRepository` for tests.
#[derive(Default)]
pub struct FakeTaskRepository {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    next_id: i64,
    tasks: Vec<Task>,
    /// Pre-loaded stale rows returned by `fetch_stale_for_notify` regardless
    /// of the query parameters. Tests set this to simulate scheduler input.
    stale_queue: Vec<StaleTaskRow>,
}

impl FakeTaskRepository {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn preload_stale(&self, rows: Vec<StaleTaskRow>) {
        let mut g = self.inner.lock().expect("FakeTaskRepository poisoned");
        g.stale_queue = rows;
    }

    pub fn snapshot(&self) -> Vec<Task> {
        let g = self.inner.lock().expect("FakeTaskRepository poisoned");
        g.tasks.clone()
    }
}

impl TaskRepository for FakeTaskRepository {
    fn insert(
        &self,
        task: NewTask,
    ) -> impl std::future::Future<Output = Result<Task, TaskRepoError>> + Send {
        let mut g = self.inner.lock().expect("FakeTaskRepository poisoned");
        if g.tasks.iter().any(|t| t.thread_id == task.thread_id) {
            return std::future::ready(Err(TaskRepoError::ThreadAlreadyMapped));
        }
        g.next_id += 1;
        let persisted = Task {
            id: TaskId(g.next_id),
            guild_id: task.guild_id,
            project_id: task.project_id,
            thread_id: task.thread_id,
            title: task.title,
            state: task.state,
            created_at: task.created_at,
            last_state_changed_at: task.created_at,
            last_notified_at: None,
            deleted_at: None,
        };
        g.tasks.push(persisted.clone());
        std::future::ready(Ok(persisted))
    }

    fn find_by_thread(
        &self,
        thread_id: ThreadId,
    ) -> impl std::future::Future<Output = Result<Option<Task>, TaskRepoError>> + Send {
        let g = self.inner.lock().expect("FakeTaskRepository poisoned");
        // Filter soft-deleted tasks — consistent with the partial unique index
        // `tasks_thread_id_unique ON tasks(thread_id) WHERE deleted_at IS NULL`.
        std::future::ready(Ok(g
            .tasks
            .iter()
            .find(|t| t.thread_id == thread_id && t.deleted_at.is_none())
            .cloned()))
    }

    fn fetch_active(
        &self,
        guild_id: GuildId,
    ) -> impl std::future::Future<Output = Result<Vec<Task>, TaskRepoError>> + Send {
        let g = self.inner.lock().expect("FakeTaskRepository poisoned");
        std::future::ready(Ok(g
            .tasks
            .iter()
            .filter(|t| t.guild_id == guild_id && t.deleted_at.is_none() && t.state.is_open())
            .cloned()
            .collect()))
    }

    fn transition_state(
        &self,
        id: TaskId,
        next: TaskState,
        at: DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<Task, TaskRepoError>> + Send {
        let mut g = self.inner.lock().expect("FakeTaskRepository poisoned");
        let Some(task) = g.tasks.iter_mut().find(|t| t.id == id) else {
            return std::future::ready(Err(TaskRepoError::NotFound));
        };
        task.state = next;
        task.last_state_changed_at = at;
        std::future::ready(Ok(task.clone()))
    }

    fn soft_delete(
        &self,
        id: TaskId,
        at: DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<(), TaskRepoError>> + Send {
        let mut g = self.inner.lock().expect("FakeTaskRepository poisoned");
        let Some(task) = g.tasks.iter_mut().find(|t| t.id == id) else {
            return std::future::ready(Err(TaskRepoError::NotFound));
        };
        task.deleted_at = Some(at);
        std::future::ready(Ok(()))
    }

    fn fetch_stale_for_notify(
        &self,
        _query: StaleQuery<'_>,
    ) -> impl std::future::Future<Output = Result<Vec<StaleTaskRow>, TaskRepoError>> + Send {
        let g = self.inner.lock().expect("FakeTaskRepository poisoned");
        std::future::ready(Ok(g.stale_queue.clone()))
    }

    fn mark_notified(
        &self,
        ids: &[TaskId],
        at: DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<(), TaskRepoError>> + Send {
        let mut g = self.inner.lock().expect("FakeTaskRepository poisoned");
        for id in ids {
            if let Some(t) = g.tasks.iter_mut().find(|t| t.id == *id) {
                t.last_notified_at = Some(at);
            }
        }
        std::future::ready(Ok(()))
    }
}
