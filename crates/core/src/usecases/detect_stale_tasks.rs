//! Stale-task detection. Returns the rows the scheduler should turn into
//! notifications. Side-effect-free aside from the underlying SQL `FOR UPDATE
//! SKIP LOCKED` performed by the repository.

use crate::domain::GuildId;
use crate::ports::{Clock, StaleQuery, StaleTaskRow, TaskRepoError, TaskRepository};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DetectStaleTasksError {
    #[error(transparent)]
    Repo(#[from] TaskRepoError),
}

pub struct DetectStaleTasks<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    repo: R,
    clock: C,
}

impl<R, C> DetectStaleTasks<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    pub fn new(repo: R, clock: C) -> Self {
        Self { repo, clock }
    }

    pub async fn execute(
        &self,
        batch_size: u32,
        guild_filter: Option<&[GuildId]>,
    ) -> Result<Vec<StaleTaskRow>, DetectStaleTasksError> {
        let q = StaleQuery {
            now: self.clock.now(),
            batch_size,
            guild_filter,
        };
        Ok(self.repo.fetch_stale_for_notify(q).await?)
    }
}
