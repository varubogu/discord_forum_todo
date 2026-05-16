use std::sync::Arc;

use chrono::Utc;
use twilight_http::Client;
use twilight_model::id::marker::{ChannelMarker, MessageMarker};
use twilight_model::id::Id;

use core_infra::SeaActiveMessageRepository;
use core_lib::domain::{ActiveMessage, ChannelId, GuildId, MessageId};
use core_lib::ports::{ActiveMessageRepoError, ActiveMessageRepository};
use discord_task::ports::{ActiveMessageService, ActiveMessageServiceError};

#[derive(Clone)]
pub struct TwilightActiveMessageService {
    client: Arc<Client>,
    repo: SeaActiveMessageRepository,
}

impl TwilightActiveMessageService {
    pub fn new(client: Arc<Client>, repo: SeaActiveMessageRepository) -> Self {
        Self { client, repo }
    }
}

impl ActiveMessageService for TwilightActiveMessageService {
    async fn sync_active_message(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
        pages: Vec<String>,
        hash: Vec<u8>,
    ) -> Result<Vec<MessageId>, ActiveMessageServiceError> {
        let hash_arr: [u8; 32] = hash
            .try_into()
            .map_err(|_| ActiveMessageServiceError::Repo("hash must be 32 bytes".into()))?;

        let existing = self.repo.find(guild_id).await.map_err(repo_err)?;

        if let Some(ref record) = existing {
            if record.last_rendered_hash == hash_arr {
                return Ok(record.message_ids.clone());
            }
        }

        let existing_ids: Vec<MessageId> = existing
            .as_ref()
            .map(|r| r.message_ids.clone())
            .unwrap_or_default();

        let ch_id = Id::<ChannelMarker>::new(channel_id.get());
        let old_count = existing_ids.len();
        let new_count = pages.len();
        let mut new_message_ids: Vec<MessageId> = Vec::with_capacity(new_count);

        for (i, page) in pages.iter().enumerate() {
            if i < old_count {
                let msg_id = Id::<MessageMarker>::new(existing_ids[i].get());
                self.client
                    .update_message(ch_id, msg_id)
                    .content(Some(page))
                    .await
                    .map_err(|e| ActiveMessageServiceError::Http(e.to_string()))?;
                new_message_ids.push(existing_ids[i]);
            } else {
                let msg = self
                    .client
                    .create_message(ch_id)
                    .content(page)
                    .await
                    .map_err(|e| ActiveMessageServiceError::Http(e.to_string()))?
                    .model()
                    .await
                    .map_err(|e| ActiveMessageServiceError::Http(e.to_string()))?;
                new_message_ids.push(MessageId::new(msg.id.get()));
            }
        }

        for old_id in existing_ids.iter().skip(new_count) {
            let msg_id = Id::<MessageMarker>::new(old_id.get());
            // Ignore delete errors (message may have already been deleted).
            let _ = self.client.delete_message(ch_id, msg_id).await;
        }

        let record = ActiveMessage::new(
            guild_id,
            channel_id,
            new_message_ids.clone(),
            hash_arr,
            Utc::now(),
        );
        self.repo.upsert(record).await.map_err(repo_err)?;

        Ok(new_message_ids)
    }
}

fn repo_err(e: ActiveMessageRepoError) -> ActiveMessageServiceError {
    ActiveMessageServiceError::Repo(e.to_string())
}
