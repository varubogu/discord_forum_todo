# 02 Data Model

PostgreSQL 16+. Schema lives in the `migration` crate, managed by `sea-orm-migration`.

## 1. ER overview

```
guilds 1───* guild_settings
 │
 ├─1───* projects
 │       │
 │       └─1───* tasks
 │
 ├─1───* channel_patterns
 ├─1───1 active_message
 ├─1───* audit_log
 └─1───* dead_letter
```

`guild_id` is a `BIGINT` storing the Discord snowflake (cast from `u64`, treated as signed because PostgreSQL has no unsigned 64-bit type; the high bit of a snowflake is never set in practice but a CHECK constraint enforces `guild_id >= 0`).

## 2. Tables

### 2.1 `guilds`

| column      | type           | notes                                  |
|-------------|----------------|----------------------------------------|
| guild_id    | BIGINT PK      | Discord snowflake.                     |
| created_at  | TIMESTAMPTZ    | DEFAULT now().                         |

### 2.2 `guild_settings`

One row per guild. All nullable except `guild_id` so we can do an upsert before any field is set.

| column                            | type          | notes                                                              |
|-----------------------------------|---------------|--------------------------------------------------------------------|
| guild_id                          | BIGINT PK FK  | references guilds.guild_id.                                        |
| user_role_id                      | BIGINT NULL   | The "user" role allowed to operate the bot.                        |
| forum_channel_id                  | BIGINT NULL   | Project forum.                                                     |
| active_channel_id                 | BIGINT NULL   | Channel hosting the active-tasks message.                          |
| notify_channel_id                 | BIGINT NULL   | Stale-task notifications go here.                                  |
| watch_category_id                 | BIGINT NULL   | Category being monitored.                                          |
| idle_threshold_not_started        | INTERVAL NULL | Default 3 days (set on first writes if NULL by the use case).      |
| idle_threshold_in_progress        | INTERVAL NULL | Default 3 days.                                                    |
| notify_cooldown                   | INTERVAL NULL | Minimum gap between two notifications per task. Default 24 hours.  |
| locale                            | TEXT NULL     | BCP-47 like `ja-JP`, `en-US`. NULL → fallback to `en-US`.          |
| updated_at                        | TIMESTAMPTZ   | DEFAULT now(); ON UPDATE handled in code.                          |
| updated_by                        | BIGINT NULL   | Discord user id who applied the last write.                        |

### 2.3 `projects`

| column            | type        | notes                                            |
|-------------------|-------------|--------------------------------------------------|
| id                | BIGSERIAL PK |                                                  |
| guild_id          | BIGINT FK   | references guilds.guild_id.                      |
| forum_channel_id  | BIGINT      | unique per guild (UNIQUE(guild_id, forum_channel_id)). |
| name              | TEXT        | display name.                                    |
| created_at        | TIMESTAMPTZ | DEFAULT now().                                   |

### 2.4 `tasks`

| column                  | type           | notes                                                  |
|-------------------------|----------------|--------------------------------------------------------|
| id                      | BIGSERIAL PK   |                                                        |
| guild_id                | BIGINT         | denormalised for fast queries.                         |
| project_id              | BIGINT FK NULL | NULL allowed for auto-generated tasks not in a project.|
| thread_id               | BIGINT UNIQUE  | Discord thread snowflake. UNIQUE NOT NULL.             |
| title                   | TEXT           | snapshot of thread name.                               |
| state                   | TEXT           | CHECK in ('not_started','in_progress','done','cancelled'). |
| created_at              | TIMESTAMPTZ    | DEFAULT now().                                         |
| last_state_changed_at   | TIMESTAMPTZ    | DEFAULT now(); written on every state change.          |
| last_notified_at        | TIMESTAMPTZ NULL | NULL until first notification.                       |
| deleted_at              | TIMESTAMPTZ NULL | soft delete.                                         |

Indexes:
- `tasks_state_changed_idx` ON (state, last_state_changed_at) WHERE deleted_at IS NULL — drives the stale-task query.
- `tasks_guild_notified_idx` ON (guild_id, last_notified_at).
- `tasks_thread_unique` UNIQUE (thread_id).

### 2.5 `channel_patterns`

| column     | type         | notes                                                  |
|------------|--------------|--------------------------------------------------------|
| id         | BIGSERIAL PK |                                                        |
| guild_id   | BIGINT FK    |                                                        |
| name       | TEXT         | display name, e.g. `ticket`.                           |
| kind       | TEXT         | CHECK in ('trigger','counter').                        |
| pattern    | TEXT         | Regex (PCRE-lite, compiled by the `regex` crate).      |
| created_at | TIMESTAMPTZ  | DEFAULT now().                                         |

UNIQUE(guild_id, name).

### 2.6 `active_message`

| column              | type           | notes                                              |
|---------------------|----------------|----------------------------------------------------|
| guild_id            | BIGINT PK      |                                                    |
| channel_id          | BIGINT         | where the message(s) live.                         |
| message_ids         | JSONB          | array of message snowflakes; may have multiple if rendering exceeds Discord limits. |
| last_rendered_hash  | BYTEA          | SHA-256 of the last rendered payload, used to skip identical edits. |
| updated_at          | TIMESTAMPTZ    | DEFAULT now().                                     |

### 2.7 `audit_log`

| column     | type           | notes                                                                  |
|------------|----------------|------------------------------------------------------------------------|
| id         | BIGSERIAL PK   |                                                                        |
| guild_id   | BIGINT         |                                                                        |
| actor_id   | BIGINT         | Discord user id who triggered the change.                              |
| action     | TEXT           | e.g. `guild_settings.set_user_role`, `task.transition`.                |
| payload    | JSONB          | free-form context.                                                     |
| created_at | TIMESTAMPTZ    | DEFAULT now().                                                         |

Index `audit_log_guild_idx` ON (guild_id, created_at DESC).

### 2.8 `dead_letter`

| column      | type         | notes                                                  |
|-------------|--------------|--------------------------------------------------------|
| id          | BIGSERIAL PK |                                                        |
| guild_id    | BIGINT       |                                                        |
| kind        | TEXT         | e.g. `notification`, `active_message_render`.          |
| payload     | JSONB        | request payload at failure.                            |
| error       | TEXT         | last error message.                                    |
| retry_count | INT          | DEFAULT 0.                                             |
| created_at  | TIMESTAMPTZ  | DEFAULT now().                                         |

## 3. Migrations naming

`migration/src/mYYYYMMDD_HHMMSS_<name>.rs`. Each migration must implement `up` and `down`; CI enforces `up → down → up` reproducibility.

## 4. Conventions

- All timestamps are `TIMESTAMPTZ` in UTC. Never `TIMESTAMP` (naive).
- IDs from Discord (`guild_id`, `channel_id`, ...) are `BIGINT`. The application layer wraps them in newtype structs.
- Soft-delete only on `tasks` (other tables are hard-delete; audit_log is append-only).
- No ON DELETE CASCADE across guilds — leaving a guild does not nuke audit_log. Cleanup is a manual operator action.

## 5. Initial data

A default row in `guild_settings` is upserted as soon as the bot joins a guild (`GuildCreate` event). All nullable fields stay NULL until configured.

## 6. Why JSONB for `message_ids`?

Discord may force us to split the active-tasks message across multiple messages (per-message char/embed limits). Storing an ordered array as JSONB lets us atomically rewrite the list when we re-render, and keeps the row count at one per guild.
