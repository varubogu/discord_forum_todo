use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait};

use core_lib::domain::{CategoryId, ChannelId, GuildId, GuildSettings, RoleId, UserId};
use core_lib::ports::{GuildSettingsRepoError, GuildSettingsRepository, GuildSettingsUpdate};

use crate::entities::guild_settings::{self, Column, Entity};

pub struct SeaGuildSettingsRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaGuildSettingsRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

fn to_db_time(dt: DateTime<Utc>) -> DateTime<FixedOffset> {
    dt.with_timezone(&FixedOffset::east_opt(0).unwrap())
}

fn from_db_time(dt: DateTime<FixedOffset>) -> DateTime<Utc> {
    dt.with_timezone(&Utc)
}

fn model_to_domain(m: guild_settings::Model) -> GuildSettings {
    GuildSettings {
        guild_id: GuildId(super::from_db_id(m.guild_id)),
        user_role_id: m.user_role_id.map(|v| RoleId(super::from_db_id(v))),
        forum_channel_id: m.forum_channel_id.map(|v| ChannelId(super::from_db_id(v))),
        active_channel_id: m.active_channel_id.map(|v| ChannelId(super::from_db_id(v))),
        notify_channel_id: m.notify_channel_id.map(|v| ChannelId(super::from_db_id(v))),
        watch_category_id: m
            .watch_category_id
            .map(|v| CategoryId(super::from_db_id(v))),
        idle_threshold_not_started: m.idle_threshold_not_started.map(chrono::Duration::seconds),
        idle_threshold_in_progress: m.idle_threshold_in_progress.map(chrono::Duration::seconds),
        notify_cooldown: m.notify_cooldown.map(chrono::Duration::seconds),
        locale: m.locale,
        updated_at: from_db_time(m.updated_at),
        updated_by: m.updated_by.map(|v| UserId(super::from_db_id(v))),
    }
}

impl GuildSettingsRepository for SeaGuildSettingsRepository {
    async fn find(
        &self,
        guild_id: GuildId,
    ) -> Result<Option<GuildSettings>, GuildSettingsRepoError> {
        Entity::find_by_id(super::to_db_id(guild_id.0))
            .one(&*self.db)
            .await
            .map(|opt| opt.map(model_to_domain))
            .map_err(|e| GuildSettingsRepoError::Backend(e.to_string()))
    }

    async fn ensure_exists(
        &self,
        guild_id: GuildId,
        now: DateTime<Utc>,
    ) -> Result<GuildSettings, GuildSettingsRepoError> {
        let new_row = guild_settings::ActiveModel {
            guild_id: Set(super::to_db_id(guild_id.0)),
            updated_at: Set(to_db_time(now)),
            ..Default::default()
        };

        let result = Entity::insert(new_row)
            .on_conflict(OnConflict::column(Column::GuildId).do_nothing().to_owned())
            .exec(&*self.db)
            .await;

        match result {
            Ok(_) | Err(DbErr::RecordNotInserted) => {}
            Err(e) => return Err(GuildSettingsRepoError::Backend(e.to_string())),
        }

        self.find(guild_id).await?.ok_or_else(|| {
            GuildSettingsRepoError::Backend("guild_settings row missing after ensure_exists".into())
        })
    }

    async fn apply(
        &self,
        guild_id: GuildId,
        update: GuildSettingsUpdate,
        actor: Option<UserId>,
        now: DateTime<Utc>,
    ) -> Result<GuildSettings, GuildSettingsRepoError> {
        let mut model = guild_settings::ActiveModel {
            guild_id: Set(super::to_db_id(guild_id.0)),
            updated_at: Set(to_db_time(now)),
            updated_by: Set(actor.map(|u| super::to_db_id(u.0))),
            ..Default::default()
        };

        match update {
            GuildSettingsUpdate::UserRole(v) => {
                model.user_role_id = Set(v.map(|id| super::to_db_id(id.0)));
            }
            GuildSettingsUpdate::ForumChannel(v) => {
                model.forum_channel_id = Set(v.map(|id| super::to_db_id(id.0)));
            }
            GuildSettingsUpdate::ActiveChannel(v) => {
                model.active_channel_id = Set(v.map(|id| super::to_db_id(id.0)));
            }
            GuildSettingsUpdate::NotifyChannel(v) => {
                model.notify_channel_id = Set(v.map(|id| super::to_db_id(id.0)));
            }
            GuildSettingsUpdate::WatchCategory(v) => {
                model.watch_category_id = Set(v.map(|id| super::to_db_id(id.0)));
            }
            GuildSettingsUpdate::IdleThresholdNotStarted(v) => {
                model.idle_threshold_not_started = Set(v.map(|d| d.num_seconds()));
            }
            GuildSettingsUpdate::IdleThresholdInProgress(v) => {
                model.idle_threshold_in_progress = Set(v.map(|d| d.num_seconds()));
            }
            GuildSettingsUpdate::NotifyCooldown(v) => {
                model.notify_cooldown = Set(v.map(|d| d.num_seconds()));
            }
            GuildSettingsUpdate::Locale(v) => {
                model.locale = Set(v);
            }
        }

        model
            .update(&*self.db)
            .await
            .map(model_to_domain)
            .map_err(|e| match e {
                DbErr::RecordNotUpdated => GuildSettingsRepoError::Backend(
                    "guild not found; call ensure_exists first".into(),
                ),
                other => GuildSettingsRepoError::Backend(other.to_string()),
            })
    }
}
