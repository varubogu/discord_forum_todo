use chrono::{TimeZone, Utc};

use core_lib::domain::{GuildId, ThreadId};
use core_lib::ports::FakeClock;
use core_lib::test_support::FakeTaskRepository;
use core_lib::usecases::CreateTaskFromThread;

#[tokio::test]
async fn creates_a_new_task_when_thread_is_unknown() {
    let repo = FakeTaskRepository::new();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap());
    let uc = CreateTaskFromThread::new(repo, clock);

    let t = uc
        .execute(GuildId(1), ThreadId(42), "hello".into())
        .await
        .unwrap();
    assert_eq!(t.guild_id, GuildId(1));
    assert_eq!(t.thread_id, ThreadId(42));
    assert_eq!(t.title, "hello");
    assert!(t.state.is_open());
}

#[tokio::test]
async fn returns_existing_task_idempotently() {
    let repo = FakeTaskRepository::new();
    let clock = FakeClock::new(Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap());
    let uc = CreateTaskFromThread::new(repo, clock);

    let first = uc
        .execute(GuildId(1), ThreadId(42), "hello".into())
        .await
        .unwrap();
    let second = uc
        .execute(GuildId(1), ThreadId(42), "different title".into())
        .await
        .unwrap();

    assert_eq!(first.id, second.id);
    assert_eq!(second.title, "hello", "title is not retroactively changed");
}
