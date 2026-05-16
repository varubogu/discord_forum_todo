use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use core_lib::domain::{ActiveMessage, ChannelId, GuildId, MessageId};
use core_lib::ports::{ActiveMessageRepoError, ActiveMessageRepository};

use crate::entities::active_messages::{self, Column, Entity};

#[derive(Clone)]
pub struct SeaActiveMessageRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaActiveMessageRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

fn to_db_time(dt: DateTime<Utc>) -> DateTime<FixedOffset> {
    dt.with_timezone(&FixedOffset::east_opt(0).unwrap())
}

fn model_to_domain(m: active_messages::Model) -> Result<ActiveMessage, ActiveMessageRepoError> {
    let ids: Vec<i64> = serde_json::from_value(m.message_ids)
        .map_err(|e| ActiveMessageRepoError::Backend(format!("bad message_ids JSON: {e}")))?;
    let message_ids: Vec<MessageId> = ids
        .into_iter()
        .map(|v| MessageId(super::from_db_id(v)))
        .collect();

    let hash: [u8; 32] = m.last_rendered_hash.try_into().map_err(|_| {
        ActiveMessageRepoError::Backend("last_rendered_hash is not 32 bytes".into())
    })?;

    Ok(ActiveMessage {
        guild_id: GuildId(super::from_db_id(m.guild_id)),
        channel_id: ChannelId(super::from_db_id(m.channel_id)),
        message_ids,
        last_rendered_hash: hash,
        updated_at: m.updated_at.with_timezone(&Utc),
    })
}

impl ActiveMessageRepository for SeaActiveMessageRepository {
    async fn find(
        &self,
        guild_id: GuildId,
    ) -> Result<Option<ActiveMessage>, ActiveMessageRepoError> {
        // active_messages has composite PK (guild_id, channel_id).
        // One record per guild — filter by guild_id alone.
        Entity::find()
            .filter(Column::GuildId.eq(super::to_db_id(guild_id.0)))
            .one(&*self.db)
            .await
            .map_err(|e| ActiveMessageRepoError::Backend(e.to_string()))
            .and_then(|opt| opt.map(model_to_domain).transpose())
    }

    async fn upsert(&self, record: ActiveMessage) -> Result<(), ActiveMessageRepoError> {
        let ids_json: serde_json::Value = serde_json::to_value(
            record
                .message_ids
                .iter()
                .map(|m| super::to_db_id(m.0))
                .collect::<Vec<_>>(),
        )
        .map_err(|e| ActiveMessageRepoError::Backend(format!("JSON serialise error: {e}")))?;

        let model = active_messages::ActiveModel {
            guild_id: Set(super::to_db_id(record.guild_id.0)),
            channel_id: Set(super::to_db_id(record.channel_id.0)),
            message_ids: Set(ids_json),
            last_rendered_hash: Set(record.last_rendered_hash.to_vec()),
            updated_at: Set(to_db_time(record.updated_at)),
        };

        Entity::insert(model)
            .on_conflict(
                OnConflict::columns([Column::GuildId, Column::ChannelId])
                    .update_columns([
                        Column::MessageIds,
                        Column::LastRenderedHash,
                        Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&*self.db)
            .await
            .map(|_| ())
            .map_err(|e| ActiveMessageRepoError::Backend(e.to_string()))
    }

    async fn delete(&self, guild_id: GuildId) -> Result<(), ActiveMessageRepoError> {
        Entity::delete_many()
            .filter(Column::GuildId.eq(super::to_db_id(guild_id.0)))
            .exec(&*self.db)
            .await
            .map(|_| ())
            .map_err(|e| ActiveMessageRepoError::Backend(e.to_string()))
    }
}
