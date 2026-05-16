mod common;

use std::sync::Arc;

use chrono::Duration;
use core_infra::{SeaGuildSettingsRepository, SeaTaskRepository};
use core_lib::domain::{ChannelId, GuildId, NewTask, RoleId, TaskState, ThreadId};
use core_lib::ports::{GuildSettingsRepository, GuildSettingsUpdate, StaleQuery, TaskRepository};

fn make_task(guild: u64, thread: u64, now: chrono::DateTime<chrono::Utc>) -> NewTask {
    NewTask::fresh(
        GuildId(guild),
        ThreadId(thread),
        format!("task-{thread}"),
        now,
    )
}

#[tokio::test]
async fn insert_and_find_by_thread() {
    let (db, _c) = common::setup_db().await;
    let db = Arc::new(db);
    let repo = SeaTaskRepository::new(db.clone());
    let now = common::now();

    let task = repo.insert(make_task(4001, 5001, now)).await.unwrap();

    let found = repo.find_by_thread(ThreadId(5001)).await.unwrap().unwrap();
    assert_eq!(found.id, task.id);
    assert_eq!(found.state, TaskState::NotStarted);
}

#[tokio::test]
async fn fetch_active_returns_only_open_tasks() {
    let (db, _c) = common::setup_db().await;
    let db = Arc::new(db);
    let repo = SeaTaskRepository::new(db.clone());
    let now = common::now();

    let t1 = repo.insert(make_task(4002, 5010, now)).await.unwrap();
    let t2 = repo.insert(make_task(4002, 5011, now)).await.unwrap();
    // transition t2 to Done
    repo.transition_state(t2.id, TaskState::Done, now)
        .await
        .unwrap();

    let active = repo.fetch_active(GuildId(4002)).await.unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, t1.id);
}

#[tokio::test]
async fn transition_state_updates_changed_at() {
    let (db, _c) = common::setup_db().await;
    let db = Arc::new(db);
    let repo = SeaTaskRepository::new(db.clone());
    let now = common::now();
    let later = now + Duration::hours(1);

    let task = repo.insert(make_task(4003, 5020, now)).await.unwrap();
    let updated = repo
        .transition_state(task.id, TaskState::InProgress, later)
        .await
        .unwrap();

    assert_eq!(updated.state, TaskState::InProgress);
    assert!(updated.last_state_changed_at >= later - Duration::seconds(1));
}

#[tokio::test]
async fn soft_delete_hides_task() {
    let (db, _c) = common::setup_db().await;
    let db = Arc::new(db);
    let repo = SeaTaskRepository::new(db.clone());
    let now = common::now();

    let task = repo.insert(make_task(4004, 5030, now)).await.unwrap();
    repo.soft_delete(task.id, now).await.unwrap();

    let found = repo.find_by_thread(ThreadId(5030)).await.unwrap();
    assert!(found.is_none(), "soft-deleted task should not be visible");
}

#[tokio::test]
async fn thread_already_mapped_on_duplicate_insert() {
    let (db, _c) = common::setup_db().await;
    let db = Arc::new(db);
    let repo = SeaTaskRepository::new(db.clone());
    let now = common::now();

    repo.insert(make_task(4005, 5040, now)).await.unwrap();
    let err = repo.insert(make_task(4005, 5040, now)).await.unwrap_err();

    assert!(
        matches!(err, core_lib::ports::TaskRepoError::ThreadAlreadyMapped),
        "expected ThreadAlreadyMapped, got {err:?}"
    );
}

#[tokio::test]
async fn fetch_stale_triggers_on_idle_task() {
    let (db, _c) = common::setup_db().await;
    let db = Arc::new(db);
    let gs_repo = SeaGuildSettingsRepository::new(db.clone());
    let task_repo = SeaTaskRepository::new(db.clone());

    let guild = GuildId(4006);
    let thread = ThreadId(5050);
    let now = common::now();

    // Set up guild with notifications enabled and a very short idle threshold (1 s)
    gs_repo.ensure_exists(guild, now).await.unwrap();
    gs_repo
        .apply(
            guild,
            GuildSettingsUpdate::UserRole(Some(RoleId(99))),
            None,
            now,
        )
        .await
        .unwrap();
    gs_repo
        .apply(
            guild,
            GuildSettingsUpdate::NotifyChannel(Some(ChannelId(88))),
            None,
            now,
        )
        .await
        .unwrap();
    gs_repo
        .apply(
            guild,
            GuildSettingsUpdate::IdleThresholdNotStarted(Some(Duration::seconds(1))),
            None,
            now,
        )
        .await
        .unwrap();

    // Insert a task and pretend it was created 2 seconds ago
    let old = now - Duration::seconds(2);
    task_repo
        .insert(NewTask {
            guild_id: guild,
            project_id: None,
            thread_id: thread,
            title: "stale".into(),
            state: TaskState::NotStarted,
            created_at: old,
        })
        .await
        .unwrap();

    let stale = task_repo
        .fetch_stale_for_notify(StaleQuery {
            now,
            batch_size: 10,
            guild_filter: None,
        })
        .await
        .unwrap();

    assert_eq!(stale.len(), 1, "should detect one stale task");
    assert_eq!(stale[0].task.thread_id, thread);
    assert_eq!(stale[0].user_role_id, RoleId(99));
    assert_eq!(stale[0].notify_channel_id, ChannelId(88));
}

#[tokio::test]
async fn mark_notified_updates_timestamp() {
    let (db, _c) = common::setup_db().await;
    let db = Arc::new(db);
    let repo = SeaTaskRepository::new(db.clone());
    let now = common::now();

    let task = repo.insert(make_task(4007, 5060, now)).await.unwrap();
    assert!(task.last_notified_at.is_none());

    repo.mark_notified(&[task.id], now).await.unwrap();

    // Re-fetch to verify
    let found = repo.find_by_thread(ThreadId(5060)).await.unwrap().unwrap();
    assert!(
        found.last_notified_at.is_some(),
        "last_notified_at should be set"
    );
}

#[tokio::test]
async fn migration_up_down_up_is_idempotent() {
    use migration::{Migrator, MigratorTrait};

    let (db, _c) = common::setup_db().await; // already ran up once
                                             // Down then up again
    Migrator::down(&db, None)
        .await
        .expect("migration down failed");
    Migrator::up(&db, None)
        .await
        .expect("migration re-up failed");
}
