use std::sync::Arc;

use twilight_http::Client;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;

use core_lib::domain::{ChannelId, RoleId, Task};
use discord_task::ports::{TaskNotifier, TaskNotifierError};

#[derive(Clone)]
pub struct TwilightTaskNotifier {
    client: Arc<Client>,
}

impl TwilightTaskNotifier {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }
}

impl TaskNotifier for TwilightTaskNotifier {
    async fn notify_stale(
        &self,
        channel_id: ChannelId,
        role_id: RoleId,
        task: &Task,
    ) -> Result<(), TaskNotifierError> {
        // Phase 8 will replace this with a Translator-based message.
        let content = format!(
            "<@&{}> タスク \"{}\" が放置されています (<#{}>)",
            role_id.get(),
            task.title,
            task.thread_id.get(),
        );

        self.client
            .create_message(Id::<ChannelMarker>::new(channel_id.get()))
            .content(&content)
            .await
            .map_err(|e| TaskNotifierError::Http(e.to_string()))?;

        Ok(())
    }
}
