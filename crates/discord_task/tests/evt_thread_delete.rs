mod common;

use core_lib::domain::{GuildId, NewTask, ThreadId};
use core_lib::ports::{FakeClock, TaskRepository};
use core_lib::test_support::FakeTaskRepository;
use discord_task::events::{GatewayEventHandler, ThreadDeleteHandler};
use twilight_gateway::Event;
use twilight_model::gateway::payload::incoming::ThreadDelete;

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap()
}

fn make_thread_delete_event(guild_id: u64, thread_id: u64) -> Event {
    let json = serde_json::json!({
        "guild_id": guild_id.to_string(),
        "id": thread_id.to_string(),
        "type": 11,       // PublicThread
        "parent_id": "3001"
    });
    let td: ThreadDelete = serde_json::from_value(json).expect("valid ThreadDelete JSON");
    Event::ThreadDelete(td)
}

/// Pre-load a task into the repo and return the handler that owns the repo.
async fn setup_with_task(
    guild: u64,
    thread: u64,
) -> ThreadDeleteHandler<FakeTaskRepository, FakeClock> {
    let repo = FakeTaskRepository::new();
    let clock = FakeClock::new(now());

    repo.insert(NewTask::fresh(
        GuildId(guild),
        ThreadId(thread),
        format!("task-{thread}"),
        now(),
    ))
    .await
    .unwrap();

    ThreadDeleteHandler::new(repo, clock)
}

#[tokio::test]
async fn thread_delete_soft_deletes_task() {
    let handler = setup_with_task(4001, 5001).await;
    let event = make_thread_delete_event(4001, 5001);

    handler.handle(&event).await.unwrap();

    // find_by_thread filters out soft-deleted tasks.
    let found = handler.repo.find_by_thread(ThreadId(5001)).await.unwrap();
    assert!(found.is_none(), "soft-deleted task must not be visible");
}

#[tokio::test]
async fn thread_delete_ignores_untracked_thread() {
    // ThreadId 9999 has no corresponding task — the handler must silently succeed.
    let repo = FakeTaskRepository::new();
    let clock = FakeClock::new(now());
    let handler = ThreadDeleteHandler::new(repo, clock);

    let event = make_thread_delete_event(4001, 9999);
    handler.handle(&event).await.unwrap(); // Must not error.
}

#[tokio::test]
async fn thread_delete_ignores_non_delete_events() {
    let handler = setup_with_task(4001, 5001).await;

    // A ThreadCreate event must be ignored; the task remains alive.
    let json = serde_json::json!({
        "id": "5001",
        "type": 11,
        "guild_id": "4001",
        "name": "task"
    });
    let channel: twilight_model::channel::Channel =
        serde_json::from_value(json).expect("valid Channel JSON");
    let event = Event::ThreadCreate(Box::new(
        twilight_model::gateway::payload::incoming::ThreadCreate(channel),
    ));

    handler.handle(&event).await.unwrap();
    // Task must still be visible (not soft-deleted).
    let found = handler.repo.find_by_thread(ThreadId(5001)).await.unwrap();
    assert!(found.is_some(), "ThreadCreate must not delete the task");
}
