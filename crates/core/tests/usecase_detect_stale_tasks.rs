use chrono::{TimeZone, Utc};

use core_lib::domain::{ChannelId, GuildId, NewTask, RoleId, TaskState, ThreadId};
use core_lib::ports::{FakeClock, StaleTaskRow, TaskRepository};
use core_lib::test_support::FakeTaskRepository;
use core_lib::usecases::DetectStaleTasks;

#[tokio::test]
async fn returns_preloaded_stale_rows() {
    let repo = FakeTaskRepository::new();
    let t0 = Utc.with_ymd_and_hms(2026, 5, 10, 0, 0, 0).unwrap();
    let task = repo
        .insert(NewTask::fresh(GuildId(1), ThreadId(7), "x", t0))
        .await
        .unwrap();
    repo.preload_stale(vec![StaleTaskRow {
        task: task.clone(),
        user_role_id: RoleId(99),
        notify_channel_id: ChannelId(20),
        locale: Some("ja-JP".into()),
    }]);

    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 5, 14, 0, 0, 0).unwrap());
    let uc = DetectStaleTasks::new(repo, clock);

    let rows = uc.execute(100, None).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].task.id, task.id);
    assert_eq!(rows[0].user_role_id, RoleId(99));
    assert!(rows[0].task.state.is_open());
    assert!(matches!(
        rows[0].task.state,
        TaskState::NotStarted | TaskState::InProgress
    ));
}
