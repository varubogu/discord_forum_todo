use chrono::{TimeZone, Utc};

use core_lib::domain::{GuildId, NewTask, TaskState, ThreadId};
use core_lib::ports::{FakeClock, TaskRepository};
use core_lib::test_support::FakeTaskRepository;
use core_lib::usecases::TransitionTaskState;

#[tokio::test]
async fn moves_task_to_new_state_and_stamps_time() {
    let repo = FakeTaskRepository::new();
    let t0 = Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap();
    let original = repo
        .insert(NewTask::fresh(GuildId(1), ThreadId(7), "x", t0))
        .await
        .unwrap();

    let t1 = Utc.with_ymd_and_hms(2026, 5, 15, 12, 0, 0).unwrap();
    let clock = FakeClock::new(t1);
    let uc = TransitionTaskState::new(repo, clock);

    let after = uc
        .execute(original.id, TaskState::InProgress)
        .await
        .unwrap();
    assert_eq!(after.state, TaskState::InProgress);
    assert_eq!(after.last_state_changed_at, t1);
}
