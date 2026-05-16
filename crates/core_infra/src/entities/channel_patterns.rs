use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "channel_patterns")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub guild_id: i64,
    pub name: String,
    pub kind: String,
    pub pattern: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
