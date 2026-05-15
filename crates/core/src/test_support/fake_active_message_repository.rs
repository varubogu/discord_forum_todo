use std::collections::HashMap;
use std::sync::Mutex;

use crate::domain::{ActiveMessage, GuildId};
use crate::ports::{ActiveMessageRepoError, ActiveMessageRepository};

#[derive(Default)]
pub struct FakeActiveMessageRepository {
    inner: Mutex<HashMap<GuildId, ActiveMessage>>,
}

impl FakeActiveMessageRepository {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ActiveMessageRepository for FakeActiveMessageRepository {
    async fn find(
        &self,
        guild_id: GuildId,
    ) -> Result<Option<ActiveMessage>, ActiveMessageRepoError> {
        let g = self.inner.lock().expect("FakeActiveMessage poisoned");
        Ok(g.get(&guild_id).cloned())
    }

    async fn upsert(&self, record: ActiveMessage) -> Result<(), ActiveMessageRepoError> {
        let mut g = self.inner.lock().expect("FakeActiveMessage poisoned");
        g.insert(record.guild_id, record);
        Ok(())
    }

    async fn delete(&self, guild_id: GuildId) -> Result<(), ActiveMessageRepoError> {
        let mut g = self.inner.lock().expect("FakeActiveMessage poisoned");
        g.remove(&guild_id);
        Ok(())
    }
}
