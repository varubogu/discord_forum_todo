//! `ProjectRepository` — forum-channel-backed project records.

use crate::domain::{ChannelId, GuildId, NewProject, Project, ProjectId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectRepoError {
    #[error("project not found")]
    NotFound,
    #[error("underlying store: {0}")]
    Backend(String),
}

pub trait ProjectRepository: Send + Sync {
    fn upsert(
        &self,
        project: NewProject,
    ) -> impl std::future::Future<Output = Result<Project, ProjectRepoError>> + Send;

    fn find_by_forum(
        &self,
        guild_id: GuildId,
        forum_channel_id: ChannelId,
    ) -> impl std::future::Future<Output = Result<Option<Project>, ProjectRepoError>> + Send;

    fn find_by_id(
        &self,
        id: ProjectId,
    ) -> impl std::future::Future<Output = Result<Option<Project>, ProjectRepoError>> + Send;
}
