use std::sync::Mutex;

use crate::domain::{ChannelId, GuildId, NewProject, Project, ProjectId};
use crate::ports::{ProjectRepoError, ProjectRepository};

#[derive(Default)]
pub struct FakeProjectRepository {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    next_id: i64,
    rows: Vec<Project>,
}

impl FakeProjectRepository {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ProjectRepository for FakeProjectRepository {
    fn upsert(
        &self,
        project: NewProject,
    ) -> impl std::future::Future<Output = Result<Project, ProjectRepoError>> + Send {
        let mut g = self.inner.lock().expect("FakeProject poisoned");
        if let Some(existing) = g.rows.iter_mut().find(|p| {
            p.guild_id == project.guild_id && p.forum_channel_id == project.forum_channel_id
        }) {
            existing.name.clone_from(&project.name);
            return std::future::ready(Ok(existing.clone()));
        }
        g.next_id += 1;
        let persisted = Project {
            id: ProjectId(g.next_id),
            guild_id: project.guild_id,
            forum_channel_id: project.forum_channel_id,
            name: project.name,
            created_at: project.created_at,
        };
        g.rows.push(persisted.clone());
        std::future::ready(Ok(persisted))
    }

    fn find_by_forum(
        &self,
        guild_id: GuildId,
        forum_channel_id: ChannelId,
    ) -> impl std::future::Future<Output = Result<Option<Project>, ProjectRepoError>> + Send {
        let g = self.inner.lock().expect("FakeProject poisoned");
        std::future::ready(Ok(g
            .rows
            .iter()
            .find(|p| p.guild_id == guild_id && p.forum_channel_id == forum_channel_id)
            .cloned()))
    }

    fn find_by_id(
        &self,
        id: ProjectId,
    ) -> impl std::future::Future<Output = Result<Option<Project>, ProjectRepoError>> + Send {
        let g = self.inner.lock().expect("FakeProject poisoned");
        std::future::ready(Ok(g.rows.iter().find(|p| p.id == id).cloned()))
    }
}
