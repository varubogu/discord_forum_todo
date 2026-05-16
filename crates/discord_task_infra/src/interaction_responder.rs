use std::sync::Arc;

use twilight_http::Client;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::{ApplicationMarker, InteractionMarker};
use twilight_model::id::Id;
use twilight_util::builder::InteractionResponseDataBuilder;

use discord_task::ports::{InteractionResponder, InteractionResponderError};

#[derive(Clone)]
pub struct TwilightInteractionResponder {
    client: Arc<Client>,
    application_id: Id<ApplicationMarker>,
}

impl TwilightInteractionResponder {
    pub fn new(client: Arc<Client>, application_id: Id<ApplicationMarker>) -> Self {
        Self {
            client,
            application_id,
        }
    }

    async fn respond(
        &self,
        interaction_id: u64,
        interaction_token: &str,
        content: &str,
        flags: Option<MessageFlags>,
    ) -> Result<(), InteractionResponderError> {
        let mut builder = InteractionResponseDataBuilder::new().content(content.to_string());
        if let Some(f) = flags {
            builder = builder.flags(f);
        }
        let data = builder.build();
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        self.client
            .interaction(self.application_id)
            .create_response(
                Id::<InteractionMarker>::new(interaction_id),
                interaction_token,
                &response,
            )
            .await
            .map_err(|e| InteractionResponderError::Http(e.to_string()))?;
        Ok(())
    }
}

impl InteractionResponder for TwilightInteractionResponder {
    async fn respond_ephemeral(
        &self,
        interaction_id: u64,
        interaction_token: &str,
        content: &str,
    ) -> Result<(), InteractionResponderError> {
        self.respond(
            interaction_id,
            interaction_token,
            content,
            Some(MessageFlags::EPHEMERAL),
        )
        .await
    }

    async fn respond_public(
        &self,
        interaction_id: u64,
        interaction_token: &str,
        content: &str,
    ) -> Result<(), InteractionResponderError> {
        self.respond(interaction_id, interaction_token, content, None)
            .await
    }
}
