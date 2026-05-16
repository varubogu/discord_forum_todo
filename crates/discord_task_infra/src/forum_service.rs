use std::sync::Arc;

use twilight_http::Client;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;

use core_lib::domain::{ChannelId, GuildId, ThreadId};
use discord_task::ports::{ForumService, ForumServiceError};

#[derive(Clone)]
pub struct TwilightForumService {
    client: Arc<Client>,
}

impl TwilightForumService {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }
}

impl ForumService for TwilightForumService {
    async fn create_thread(
        &self,
        _guild_id: GuildId,
        forum_channel_id: ChannelId,
        title: &str,
    ) -> Result<ThreadId, ForumServiceError> {
        let channel_id = Id::<ChannelMarker>::new(forum_channel_id.get());

        // Forum threads require an initial message; use a zero-width space as minimal content.
        let thread = self
            .client
            .create_forum_thread(channel_id, title)
            .message()
            .content("\u{200b}")
            .await
            .map_err(|e| ForumServiceError::Http(e.to_string()))?
            .model()
            .await
            .map_err(|e| ForumServiceError::Http(e.to_string()))?;

        Ok(ThreadId::new(thread.channel.id.get()))
    }
}
