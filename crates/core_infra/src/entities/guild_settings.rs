use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "guild_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    pub user_role_id: Option<i64>,
    pub forum_channel_id: Option<i64>,
    pub active_channel_id: Option<i64>,
    pub notify_channel_id: Option<i64>,
    pub watch_category_id: Option<i64>,
    /// Seconds
    pub idle_threshold_not_started: Option<i64>,
    /// Seconds
    pub idle_threshold_in_progress: Option<i64>,
    /// Seconds
    pub notify_cooldown: Option<i64>,
    pub locale: Option<String>,
    pub updated_at: DateTimeWithTimeZone,
    pub updated_by: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
