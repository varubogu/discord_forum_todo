//! Create a `Task` from a newly observed forum thread. Idempotent — if the
//! thread already maps to a task we return the existing one untouched.

use crate::domain::{GuildId, NewTask, Task, ThreadId};
use crate::ports::{Clock, TaskRepoError, TaskRepository};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateTaskFromThreadError {
    #[error(transparent)]
    Repo(#[from] TaskRepoError),
}

pub struct CreateTaskFromThread<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    /// The task repository. Public so downstream adapter tests can inspect state.
    pub repo: R,
    pub clock: C,
}

impl<R, C> CreateTaskFromThread<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    pub fn new(repo: R, clock: C) -> Self {
        Self { repo, clock }
    }

    pub async fn execute(
        &self,
        guild_id: GuildId,
        thread_id: ThreadId,
        title: String,
    ) -> Result<Task, CreateTaskFromThreadError> {
        if let Some(existing) = self.repo.find_by_thread(thread_id).await? {
            return Ok(existing);
        }
        let now = self.clock.now();
        let new_task = NewTask::fresh(guild_id, thread_id, title, now);
        match self.repo.insert(new_task).await {
            Ok(t) => Ok(t),
            // Lost the race with another instance — fetch and return.
            Err(TaskRepoError::ThreadAlreadyMapped) => self
                .repo
                .find_by_thread(thread_id)
                .await?
                .ok_or(CreateTaskFromThreadError::Repo(TaskRepoError::NotFound)),
            Err(e) => Err(e.into()),
        }
    }
}
