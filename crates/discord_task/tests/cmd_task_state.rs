mod common;

use core_lib::domain::{GuildId, NewTask, TaskId, TaskState, ThreadId};
use core_lib::ports::{FakeClock, TaskRepository};
use core_lib::test_support::FakeTaskRepository;
use discord_task::commands::{TaskStateHandler, TaskStateInput};

use common::FakeInteractionResponder;

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap()
}

/// Insert a task and return its database ID.
async fn insert_task(repo: &FakeTaskRepository, guild: u64, thread: u64) -> TaskId {
    let task = repo
        .insert(NewTask::fresh(
            GuildId(guild),
            ThreadId(thread),
            format!("task-{thread}"),
            now(),
        ))
        .await
        .unwrap();
    task.id
}

#[tokio::test]
async fn task_state_transitions_to_done_and_replies_publicly() {
    let repo = FakeTaskRepository::new();
    let task_id = insert_task(&repo, 4001, 5001).await;

    let clock = FakeClock::new(now());
    let responder = FakeInteractionResponder::default();
    let handler = TaskStateHandler::new(repo, clock, responder);

    handler
        .handle(TaskStateInput {
            task_id,
            next_state: TaskState::Done,
            interaction_id: 99,
            interaction_token: "tok".to_string(),
        })
        .await
        .unwrap();

    let responses = handler.responder.all();
    assert_eq!(responses.len(), 1);
    assert!(!responses[0].ephemeral, "task state reply must be public");
    assert!(
        responses[0].content.contains("done"),
        "reply must mention the new state"
    );
}

#[tokio::test]
async fn task_state_fails_for_nonexistent_task() {
    let repo = FakeTaskRepository::new();
    let clock = FakeClock::new(now());
    let responder = FakeInteractionResponder::default();
    let handler = TaskStateHandler::new(repo, clock, responder);

    let result = handler
        .handle(TaskStateInput {
            task_id: TaskId::new(9999),
            next_state: TaskState::InProgress,
            interaction_id: 1,
            interaction_token: "t".to_string(),
        })
        .await;

    assert!(result.is_err(), "transitioning nonexistent task must fail");
    // No response should have been sent.
    assert!(handler.responder.all().is_empty());
}
