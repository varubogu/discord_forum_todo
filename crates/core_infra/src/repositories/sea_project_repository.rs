use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use core_lib::domain::{ChannelId, GuildId, NewProject, Project, ProjectId};
use core_lib::ports::{ProjectRepoError, ProjectRepository};

use crate::entities::projects::{self, Column, Entity};

pub struct SeaProjectRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaProjectRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

fn to_db_time(dt: DateTime<Utc>) -> DateTime<FixedOffset> {
    dt.with_timezone(&FixedOffset::east_opt(0).unwrap())
}

fn model_to_domain(m: projects::Model) -> Project {
    Project {
        id: ProjectId::new(m.id),
        guild_id: GuildId(super::from_db_id(m.guild_id)),
        forum_channel_id: ChannelId(super::from_db_id(m.forum_channel_id)),
        name: m.name,
        created_at: m.created_at.with_timezone(&Utc),
    }
}

impl ProjectRepository for SeaProjectRepository {
    async fn upsert(&self, project: NewProject) -> Result<Project, ProjectRepoError> {
        let model = projects::ActiveModel {
            guild_id: Set(super::to_db_id(project.guild_id.0)),
            forum_channel_id: Set(super::to_db_id(project.forum_channel_id.0)),
            name: Set(project.name),
            created_at: Set(to_db_time(project.created_at)),
            ..Default::default()
        };

        Entity::insert(model)
            .on_conflict(
                OnConflict::columns([Column::GuildId, Column::ForumChannelId])
                    .update_columns([Column::Name])
                    .to_owned(),
            )
            .exec_with_returning(&*self.db)
            .await
            .map(model_to_domain)
            .map_err(|e| ProjectRepoError::Backend(e.to_string()))
    }

    async fn find_by_forum(
        &self,
        guild_id: GuildId,
        forum_channel_id: ChannelId,
    ) -> Result<Option<Project>, ProjectRepoError> {
        Entity::find()
            .filter(Column::GuildId.eq(super::to_db_id(guild_id.0)))
            .filter(Column::ForumChannelId.eq(super::to_db_id(forum_channel_id.0)))
            .one(&*self.db)
            .await
            .map(|opt| opt.map(model_to_domain))
            .map_err(|e| ProjectRepoError::Backend(e.to_string()))
    }

    async fn find_by_id(&self, id: ProjectId) -> Result<Option<Project>, ProjectRepoError> {
        Entity::find_by_id(id.get())
            .one(&*self.db)
            .await
            .map(|opt| opt.map(model_to_domain))
            .map_err(|e| ProjectRepoError::Backend(e.to_string()))
    }
}
