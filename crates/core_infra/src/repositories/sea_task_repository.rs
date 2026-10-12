use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbBackend, DbErr,
    EntityTrait, FromQueryResult, QueryFilter, Statement,
};

use core_lib::domain::{ChannelId, GuildId, NewTask, RoleId, Task, TaskId, TaskState, ThreadId};
use core_lib::ports::{StaleQuery, StaleTaskRow, TaskRepoError, TaskRepository};

use crate::entities::tasks::{self, Column, Entity};

pub struct SeaTaskRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaTaskRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

fn to_db_time(dt: DateTime<Utc>) -> DateTime<FixedOffset> {
    dt.with_timezone(&FixedOffset::east_opt(0).unwrap())
}

fn model_to_task(m: tasks::Model) -> Result<Task, TaskRepoError> {
    let state = TaskState::parse(&m.state)
        .ok_or_else(|| TaskRepoError::Backend(format!("invalid task state: {}", m.state)))?;
    Ok(Task {
        id: TaskId::new(m.id),
        guild_id: GuildId(super::from_db_id(m.guild_id)),
        project_id: m.project_id,
        thread_id: ThreadId(super::from_db_id(m.thread_id)),
        title: m.title,
        state,
        created_at: m.created_at.with_timezone(&Utc),
        last_state_changed_at: m.last_state_changed_at.with_timezone(&Utc),
        last_notified_at: m.last_notified_at.map(|dt| dt.with_timezone(&Utc)),
        deleted_at: m.deleted_at.map(|dt| dt.with_timezone(&Utc)),
    })
}

fn is_unique_violation(err: &DbErr) -> bool {
    err.to_string().contains("duplicate key") || err.to_string().contains("unique constraint")
}

#[derive(Debug, FromQueryResult)]
struct StaleRow {
    id: i64,
    guild_id: i64,
    project_id: Option<i64>,
    thread_id: i64,
    title: String,
    state: String,
    created_at: DateTime<FixedOffset>,
    last_state_changed_at: DateTime<FixedOffset>,
    last_notified_at: Option<DateTime<FixedOffset>>,
    deleted_at: Option<DateTime<FixedOffset>>,
    user_role_id: i64,
    notify_channel_id: i64,
    locale: Option<String>,
}

impl TaskRepository for SeaTaskRepository {
    async fn insert(&self, task: NewTask) -> Result<Task, TaskRepoError> {
        let model = tasks::ActiveModel {
            guild_id: Set(super::to_db_id(task.guild_id.0)),
            project_id: Set(task.project_id),
            thread_id: Set(super::to_db_id(task.thread_id.0)),
            title: Set(task.title),
            state: Set(task.state.as_str().to_owned()),
            created_at: Set(to_db_time(task.created_at)),
            last_state_changed_at: Set(to_db_time(task.created_at)),
            ..Default::default()
        };

        Entity::insert(model)
            .exec_with_returning(&*self.db)
            .await
            .map_err(|e| {
                if is_unique_violation(&e) {
                    TaskRepoError::ThreadAlreadyMapped
                } else {
                    TaskRepoError::Backend(e.to_string())
                }
            })
            .and_then(model_to_task)
    }

    async fn find_by_thread(&self, thread_id: ThreadId) -> Result<Option<Task>, TaskRepoError> {
        Entity::find()
            .filter(Column::ThreadId.eq(super::to_db_id(thread_id.0)))
            .filter(Column::DeletedAt.is_null())
            .one(&*self.db)
            .await
            .map_err(|e| TaskRepoError::Backend(e.to_string()))
            .and_then(|opt| opt.map(model_to_task).transpose())
    }

    async fn fetch_active(&self, guild_id: GuildId) -> Result<Vec<Task>, TaskRepoError> {
        Entity::find()
            .filter(Column::GuildId.eq(super::to_db_id(guild_id.0)))
            .filter(Column::State.is_in([
                TaskState::NotStarted.as_str(),
                TaskState::InProgress.as_str(),
            ]))
            .filter(Column::DeletedAt.is_null())
            .all(&*self.db)
            .await
            .map_err(|e| TaskRepoError::Backend(e.to_string()))
            .and_then(|rows| rows.into_iter().map(model_to_task).collect())
    }

    async fn transition_state(
        &self,
        id: TaskId,
        next: TaskState,
        at: DateTime<Utc>,
    ) -> Result<Task, TaskRepoError> {
        let model = tasks::ActiveModel {
            id: Set(id.get()),
            state: Set(next.as_str().to_owned()),
            last_state_changed_at: Set(to_db_time(at)),
            ..Default::default()
        };

        model
            .update(&*self.db)
            .await
            .map_err(|e| match e {
                DbErr::RecordNotUpdated => TaskRepoError::NotFound,
                other => TaskRepoError::Backend(other.to_string()),
            })
            .and_then(model_to_task)
    }

    async fn soft_delete(&self, id: TaskId, at: DateTime<Utc>) -> Result<(), TaskRepoError> {
        let model = tasks::ActiveModel {
            id: Set(id.get()),
            deleted_at: Set(Some(to_db_time(at))),
            ..Default::default()
        };

        model
            .update(&*self.db)
            .await
            .map(|_| ())
            .map_err(|e| match e {
                DbErr::RecordNotUpdated => TaskRepoError::NotFound,
                other => TaskRepoError::Backend(other.to_string()),
            })
    }

    async fn fetch_stale_for_notify(
        &self,
        query: StaleQuery<'_>,
    ) -> Result<Vec<StaleTaskRow>, TaskRepoError> {
        let now = query.now;
        let batch_size = query.batch_size;
        let guild_ids: Option<Vec<i64>> = query
            .guild_filter
            .map(|s| s.iter().map(|g| super::to_db_id(g.0)).collect());

        let guild_filter_clause = if let Some(ref guilds) = guild_ids {
            let placeholders: Vec<String> = guilds
                .iter()
                .enumerate()
                .map(|(i, _)| format!("${}", i + 3))
                .collect();
            format!("AND t.guild_id IN ({})", placeholders.join(", "))
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT \
                 t.id, t.guild_id, t.project_id, t.thread_id, t.title, t.state, \
                 t.created_at, t.last_state_changed_at, t.last_notified_at, t.deleted_at, \
                 gs.user_role_id, gs.notify_channel_id, gs.locale \
             FROM tasks t \
             JOIN guild_settings gs ON gs.guild_id = t.guild_id \
             WHERE t.deleted_at IS NULL \
               AND t.state IN ('not_started', 'in_progress') \
               AND ( \
                 (t.state = 'not_started' \
                  AND t.last_state_changed_at < \
                      $1 - (COALESCE(gs.idle_threshold_not_started, 259200) * interval '1 second')) \
                 OR \
                 (t.state = 'in_progress' \
                  AND t.last_state_changed_at < \
                      $1 - (COALESCE(gs.idle_threshold_in_progress, 259200) * interval '1 second')) \
               ) \
               AND ( \
                 t.last_notified_at IS NULL \
                 OR t.last_notified_at < $1 - (COALESCE(gs.notify_cooldown, 86400) * interval '1 second') \
               ) \
               AND gs.user_role_id IS NOT NULL \
               AND gs.notify_channel_id IS NOT NULL \
               {guild_filter_clause} \
             ORDER BY t.last_state_changed_at \
             LIMIT $2 \
             FOR UPDATE OF t SKIP LOCKED"
        );

        let mut values: Vec<sea_orm::Value> = vec![
            sea_orm::Value::ChronoDateTimeWithTimeZone(Some(
                now.with_timezone(&FixedOffset::east_opt(0).unwrap()),
            )),
            sea_orm::Value::BigInt(Some(i64::from(batch_size))),
        ];
        if let Some(guilds) = guild_ids {
            for g in guilds {
                values.push(sea_orm::Value::BigInt(Some(g)));
            }
        }

        let stmt = Statement::from_sql_and_values(DbBackend::Postgres, &sql, values);
        StaleRow::find_by_statement(stmt)
            .all(&*self.db)
            .await
            .map_err(|e| TaskRepoError::Backend(e.to_string()))?
            .into_iter()
            .map(|row| {
                let state = TaskState::parse(&row.state).ok_or_else(|| {
                    TaskRepoError::Backend(format!("invalid task state: {}", row.state))
                })?;
                Ok(StaleTaskRow {
                    task: Task {
                        id: TaskId::new(row.id),
                        guild_id: GuildId(super::from_db_id(row.guild_id)),
                        project_id: row.project_id,
                        thread_id: ThreadId(super::from_db_id(row.thread_id)),
                        title: row.title,
                        state,
                        created_at: row.created_at.with_timezone(&Utc),
                        last_state_changed_at: row.last_state_changed_at.with_timezone(&Utc),
                        last_notified_at: row.last_notified_at.map(|dt| dt.with_timezone(&Utc)),
                        deleted_at: row.deleted_at.map(|dt| dt.with_timezone(&Utc)),
                    },
                    user_role_id: RoleId(super::from_db_id(row.user_role_id)),
                    notify_channel_id: ChannelId(super::from_db_id(row.notify_channel_id)),
                    locale: row.locale,
                })
            })
            .collect()
    }

    async fn mark_notified(&self, ids: &[TaskId], at: DateTime<Utc>) -> Result<(), TaskRepoError> {
        if ids.is_empty() {
            return Ok(());
        }
        let id_vals: Vec<i64> = ids.iter().map(|id| id.get()).collect();
        Entity::update_many()
            .col_expr(Column::LastNotifiedAt, Expr::value(to_db_time(at)))
            .filter(Column::Id.is_in(id_vals))
            .exec(&*self.db)
            .await
            .map(|_| ())
            .map_err(|e| TaskRepoError::Backend(e.to_string()))
    }
}
