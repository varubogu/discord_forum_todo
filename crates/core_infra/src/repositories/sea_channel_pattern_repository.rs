use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use core_lib::domain::{
    ChannelPattern, ChannelPatternId, ChannelPatternKind, GuildId, NewChannelPattern,
};
use core_lib::ports::{ChannelPatternRepoError, ChannelPatternRepository};

use crate::entities::channel_patterns::{self, Column, Entity};

pub struct SeaChannelPatternRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaChannelPatternRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

fn to_db_time(dt: DateTime<Utc>) -> DateTime<FixedOffset> {
    dt.with_timezone(&FixedOffset::east_opt(0).unwrap())
}

fn model_to_domain(m: channel_patterns::Model) -> Result<ChannelPattern, ChannelPatternRepoError> {
    let kind = ChannelPatternKind::parse(&m.kind).ok_or_else(|| {
        ChannelPatternRepoError::Backend(format!("invalid pattern kind: {}", m.kind))
    })?;
    Ok(ChannelPattern {
        id: ChannelPatternId::new(m.id),
        guild_id: GuildId(super::from_db_id(m.guild_id)),
        name: m.name,
        kind,
        pattern: m.pattern,
        created_at: m.created_at.with_timezone(&Utc),
    })
}

fn is_unique_violation(err: &DbErr) -> bool {
    err.to_string().contains("duplicate key") || err.to_string().contains("unique constraint")
}

impl ChannelPatternRepository for SeaChannelPatternRepository {
    async fn list(
        &self,
        guild_id: GuildId,
    ) -> Result<Vec<ChannelPattern>, ChannelPatternRepoError> {
        Entity::find()
            .filter(Column::GuildId.eq(super::to_db_id(guild_id.0)))
            .all(&*self.db)
            .await
            .map_err(|e| ChannelPatternRepoError::Backend(e.to_string()))
            .and_then(|rows| rows.into_iter().map(model_to_domain).collect())
    }

    async fn insert(
        &self,
        pattern: NewChannelPattern,
    ) -> Result<ChannelPattern, ChannelPatternRepoError> {
        let name_for_err = pattern.name.clone();
        let model = channel_patterns::ActiveModel {
            guild_id: Set(super::to_db_id(pattern.guild_id.0)),
            name: Set(pattern.name),
            kind: Set(pattern.kind.as_str().to_owned()),
            pattern: Set(pattern.pattern),
            created_at: Set(to_db_time(pattern.created_at)),
            ..Default::default()
        };

        Entity::insert(model)
            .exec_with_returning(&*self.db)
            .await
            .map_err(|e| {
                if is_unique_violation(&e) {
                    ChannelPatternRepoError::DuplicateName(name_for_err)
                } else {
                    ChannelPatternRepoError::Backend(e.to_string())
                }
            })
            .and_then(model_to_domain)
    }

    async fn delete_by_name(
        &self,
        guild_id: GuildId,
        name: &str,
    ) -> Result<(), ChannelPatternRepoError> {
        let result = Entity::delete_many()
            .filter(Column::GuildId.eq(super::to_db_id(guild_id.0)))
            .filter(Column::Name.eq(name))
            .exec(&*self.db)
            .await
            .map_err(|e| ChannelPatternRepoError::Backend(e.to_string()))?;

        if result.rows_affected == 0 {
            return Err(ChannelPatternRepoError::NotFound);
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: ChannelPatternId,
    ) -> Result<Option<ChannelPattern>, ChannelPatternRepoError> {
        Entity::find_by_id(id.0)
            .one(&*self.db)
            .await
            .map_err(|e| ChannelPatternRepoError::Backend(e.to_string()))
            .and_then(|opt| opt.map(model_to_domain).transpose())
    }
}
