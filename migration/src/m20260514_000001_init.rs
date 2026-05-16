use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20260514_000001_init"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    #[allow(clippy::too_many_lines)]
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. guild_settings
        manager
            .create_table(
                Table::create()
                    .table(GuildSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GuildSettings::GuildId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GuildSettings::UserRoleId).big_integer())
                    .col(ColumnDef::new(GuildSettings::ForumChannelId).big_integer())
                    .col(ColumnDef::new(GuildSettings::ActiveChannelId).big_integer())
                    .col(ColumnDef::new(GuildSettings::NotifyChannelId).big_integer())
                    .col(ColumnDef::new(GuildSettings::WatchCategoryId).big_integer())
                    .col(ColumnDef::new(GuildSettings::IdleThresholdNotStarted).big_integer())
                    .col(ColumnDef::new(GuildSettings::IdleThresholdInProgress).big_integer())
                    .col(ColumnDef::new(GuildSettings::NotifyCooldown).big_integer())
                    .col(ColumnDef::new(GuildSettings::Locale).string_len(10))
                    .col(
                        ColumnDef::new(GuildSettings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(GuildSettings::UpdatedBy).big_integer())
                    .to_owned(),
            )
            .await?;

        // 2. projects
        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Projects::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Projects::GuildId).big_integer().not_null())
                    .col(
                        ColumnDef::new(Projects::ForumChannelId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Projects::Name).text().not_null())
                    .col(
                        ColumnDef::new(Projects::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Projects::Table)
                    .name("projects_guild_forum_unique")
                    .col(Projects::GuildId)
                    .col(Projects::ForumChannelId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 3. tasks
        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tasks::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tasks::GuildId).big_integer().not_null())
                    .col(ColumnDef::new(Tasks::ProjectId).big_integer())
                    .col(ColumnDef::new(Tasks::ThreadId).big_integer().not_null())
                    .col(ColumnDef::new(Tasks::Title).text().not_null())
                    .col(
                        ColumnDef::new(Tasks::State)
                            .string_len(20)
                            .not_null()
                            .default("not_started"),
                    )
                    .col(
                        ColumnDef::new(Tasks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Tasks::LastStateChangedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Tasks::LastNotifiedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Tasks::DeletedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Tasks::Table, Tasks::ProjectId)
                            .to(Projects::Table, Projects::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // partial unique: thread_id WHERE deleted_at IS NULL
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE UNIQUE INDEX tasks_thread_id_unique \
                 ON tasks(thread_id) WHERE deleted_at IS NULL",
            )
            .await?;

        // index for stale detection
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX tasks_state_changed \
                 ON tasks(state, last_state_changed_at) WHERE deleted_at IS NULL",
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Tasks::Table)
                    .name("tasks_guild_notified")
                    .col(Tasks::GuildId)
                    .col(Tasks::LastNotifiedAt)
                    .to_owned(),
            )
            .await?;

        // 4. channel_patterns
        manager
            .create_table(
                Table::create()
                    .table(ChannelPatterns::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChannelPatterns::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ChannelPatterns::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ChannelPatterns::Name).text().not_null())
                    .col(
                        ColumnDef::new(ChannelPatterns::Kind)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ChannelPatterns::Pattern).text().not_null())
                    .col(
                        ColumnDef::new(ChannelPatterns::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(ChannelPatterns::Table)
                    .name("channel_patterns_guild_name_unique")
                    .col(ChannelPatterns::GuildId)
                    .col(ChannelPatterns::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 5. active_messages
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS active_messages ( \
                     guild_id           BIGINT NOT NULL, \
                     channel_id         BIGINT NOT NULL, \
                     message_ids        JSONB  NOT NULL, \
                     last_rendered_hash BYTEA  NOT NULL, \
                     updated_at         TIMESTAMPTZ NOT NULL, \
                     PRIMARY KEY (guild_id, channel_id) \
                 )",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS active_messages")
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(ChannelPatterns::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Tasks::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Projects::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(GuildSettings::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// ─── Iden enums ──────────────────────────────────────────────────────────────

#[derive(Iden)]
enum GuildSettings {
    Table,
    GuildId,
    UserRoleId,
    ForumChannelId,
    ActiveChannelId,
    NotifyChannelId,
    WatchCategoryId,
    IdleThresholdNotStarted,
    IdleThresholdInProgress,
    NotifyCooldown,
    Locale,
    UpdatedAt,
    UpdatedBy,
}

#[derive(Iden)]
enum Projects {
    Table,
    Id,
    GuildId,
    ForumChannelId,
    Name,
    CreatedAt,
}

#[derive(Iden)]
enum Tasks {
    Table,
    Id,
    GuildId,
    ProjectId,
    ThreadId,
    Title,
    State,
    CreatedAt,
    LastStateChangedAt,
    LastNotifiedAt,
    DeletedAt,
}

#[derive(Iden)]
enum ChannelPatterns {
    Table,
    Id,
    GuildId,
    Name,
    Kind,
    Pattern,
    CreatedAt,
}
