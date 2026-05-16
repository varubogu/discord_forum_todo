//! Move a task to a new state, validating the transition and stamping the
//! `last_state_changed_at` timestamp.

use crate::domain::{Task, TaskId, TaskState};
use crate::errors::DomainError;
use crate::ports::{Clock, TaskRepoError, TaskRepository};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransitionTaskStateError {
    #[error(transparent)]
    Repo(#[from] TaskRepoError),
    #[error(transparent)]
    Domain(#[from] DomainError),
}

pub struct TransitionTaskState<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    /// The task repository. Public so downstream adapter tests can inspect state.
    pub repo: R,
    pub clock: C,
}

impl<R, C> TransitionTaskState<R, C>
where
    R: TaskRepository,
    C: Clock,
{
    pub fn new(repo: R, clock: C) -> Self {
        Self { repo, clock }
    }

    pub async fn execute(
        &self,
        id: TaskId,
        next: TaskState,
    ) -> Result<Task, TransitionTaskStateError> {
        // We could load + check first, but transitioning to the same state is
        // a no-op the repo can also reject. The double-fetch keeps the audit
        // trail simple: one read, then one write.
        let now = self.clock.now();
        let task = self.repo.transition_state(id, next, now).await?;
        if !task.state.eq(&next) {
            // Defensive: an adapter that returned a state mismatch is buggy.
            return Err(TransitionTaskStateError::Domain(
                DomainError::InvalidStateTransition,
            ));
        }
        Ok(task)
    }
}
