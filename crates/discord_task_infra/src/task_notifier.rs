use std::sync::Arc;

use twilight_http::Client;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;

use core_lib::domain::{ChannelId, RoleId, Task};
use core_lib::i18n::{Locale, Translator};
use core_lib::tr;
use discord_task::ports::{TaskNotifier, TaskNotifierError};

pub struct TwilightTaskNotifier<T: Translator> {
    client: Arc<Client>,
    translator: Arc<T>,
}

impl<T: Translator> TwilightTaskNotifier<T> {
    pub fn new(client: Arc<Client>, translator: Arc<T>) -> Self {
        Self { client, translator }
    }
}

impl<T: Translator> Clone for TwilightTaskNotifier<T> {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            translator: Arc::clone(&self.translator),
        }
    }
}

impl<T: Translator> TaskNotifier for TwilightTaskNotifier<T> {
    async fn notify_stale(
        &self,
        channel_id: ChannelId,
        role_id: RoleId,
        task: &Task,
        locale: &Locale,
    ) -> Result<(), TaskNotifierError> {
        let role_str = role_id.get().to_string();
        let thread_str = task.thread_id.get().to_string();
        let content = tr!(
            self.translator,
            locale,
            "notification-stale-task",
            role = role_str.as_str(),
            title = task.title.as_str(),
            thread = thread_str.as_str(),
        );

        self.client
            .create_message(Id::<ChannelMarker>::new(channel_id.get()))
            .content(&content)
            .await
            .map_err(|e| TaskNotifierError::Http(e.to_string()))?;

        Ok(())
    }
}
