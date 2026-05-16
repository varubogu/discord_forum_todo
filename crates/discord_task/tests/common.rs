//! Shared in-memory fakes for Discord-specific ports.
#![allow(dead_code)]

use std::sync::Mutex;

use core_lib::domain::{ChannelId, GuildId, MessageId, RoleId, Task, ThreadId};
use core_lib::i18n::Locale;
use discord_task::ports::{
    ActiveMessageService, ActiveMessageServiceError, ChannelInspector, ChannelInspectorError,
    ForumService, ForumServiceError, InteractionResponder, InteractionResponderError, TaskNotifier,
    TaskNotifierError,
};

// ── FakeForumService ─────────────────────────────────────────────────────────

/// Records `create_thread` calls and always returns the same pre-configured thread ID.
pub struct FakeForumService {
    pub next_thread_id: u64,
    pub calls: Mutex<Vec<(GuildId, ChannelId, String)>>,
}

impl FakeForumService {
    pub fn new(next_thread_id: u64) -> Self {
        Self {
            next_thread_id,
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn recorded_calls(&self) -> Vec<(GuildId, ChannelId, String)> {
        self.calls.lock().unwrap().clone()
    }
}

impl ForumService for FakeForumService {
    async fn create_thread(
        &self,
        guild_id: GuildId,
        forum_channel_id: ChannelId,
        title: &str,
    ) -> Result<ThreadId, ForumServiceError> {
        self.calls
            .lock()
            .unwrap()
            .push((guild_id, forum_channel_id, title.to_string()));
        Ok(ThreadId(self.next_thread_id))
    }
}

// ── FakeChannelInspector ──────────────────────────────────────────────────────

/// Returns a fixed count for every `count_matching_channels` call.
pub struct FakeChannelInspector {
    pub count: u64,
}

impl FakeChannelInspector {
    pub fn new(count: u64) -> Self {
        Self { count }
    }
}

impl ChannelInspector for FakeChannelInspector {
    async fn count_matching_channels(
        &self,
        _guild_id: GuildId,
        _pattern: &str,
    ) -> Result<u64, ChannelInspectorError> {
        Ok(self.count)
    }
}

// ── FakeActiveMessageService ──────────────────────────────────────────────────

/// Records `sync_active_message` calls and returns empty message-id list.
#[derive(Default)]
pub struct FakeActiveMessageService {
    pub calls: Mutex<u32>,
}

impl ActiveMessageService for FakeActiveMessageService {
    async fn sync_active_message(
        &self,
        _guild_id: GuildId,
        _channel_id: ChannelId,
        _pages: Vec<String>,
        _hash: Vec<u8>,
    ) -> Result<Vec<MessageId>, ActiveMessageServiceError> {
        *self.calls.lock().unwrap() += 1;
        Ok(vec![])
    }
}

// ── FakeTaskNotifier ─────────────────────────────────────────────────────────

/// Records `notify_stale` calls.
#[derive(Default)]
pub struct FakeTaskNotifier {
    pub calls: Mutex<Vec<(ChannelId, RoleId)>>,
}

impl TaskNotifier for FakeTaskNotifier {
    async fn notify_stale(
        &self,
        channel_id: ChannelId,
        role_id: RoleId,
        _task: &Task,
        _locale: &Locale,
    ) -> Result<(), TaskNotifierError> {
        self.calls.lock().unwrap().push((channel_id, role_id));
        Ok(())
    }
}

// ── FakeInteractionResponder ──────────────────────────────────────────────────

/// Records all interaction responses.
#[derive(Default)]
pub struct FakeInteractionResponder {
    pub responses: Mutex<Vec<Response>>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub interaction_id: u64,
    pub ephemeral: bool,
    pub content: String,
}

impl FakeInteractionResponder {
    pub fn all(&self) -> Vec<Response> {
        self.responses.lock().unwrap().clone()
    }
}

impl InteractionResponder for FakeInteractionResponder {
    async fn respond_ephemeral(
        &self,
        interaction_id: u64,
        _token: &str,
        content: &str,
    ) -> Result<(), InteractionResponderError> {
        self.responses.lock().unwrap().push(Response {
            interaction_id,
            ephemeral: true,
            content: content.to_string(),
        });
        Ok(())
    }

    async fn respond_public(
        &self,
        interaction_id: u64,
        _token: &str,
        content: &str,
    ) -> Result<(), InteractionResponderError> {
        self.responses.lock().unwrap().push(Response {
            interaction_id,
            ephemeral: false,
            content: content.to_string(),
        });
        Ok(())
    }
}
