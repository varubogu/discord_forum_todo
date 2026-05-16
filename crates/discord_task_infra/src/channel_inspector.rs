use std::sync::Arc;

use regex::Regex;
use twilight_http::Client;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use core_lib::domain::GuildId;
use discord_task::ports::{ChannelInspector, ChannelInspectorError};

#[derive(Clone)]
pub struct TwilightChannelInspector {
    client: Arc<Client>,
}

impl TwilightChannelInspector {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }
}

impl ChannelInspector for TwilightChannelInspector {
    async fn count_matching_channels(
        &self,
        guild_id: GuildId,
        pattern: &str,
    ) -> Result<u64, ChannelInspectorError> {
        let Ok(re) = Regex::new(pattern) else {
            return Ok(0);
        };

        let channels = self
            .client
            .guild_channels(Id::<GuildMarker>::new(guild_id.get()))
            .await
            .map_err(|e| ChannelInspectorError::Http(e.to_string()))?
            .models()
            .await
            .map_err(|e| ChannelInspectorError::Http(e.to_string()))?;

        let count = channels
            .iter()
            .filter(|ch| ch.name.as_deref().is_some_and(|n| re.is_match(n)))
            .count();

        Ok(u64::try_from(count).unwrap_or(u64::MAX))
    }
}
