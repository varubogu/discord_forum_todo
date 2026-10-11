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
    fn find(
        &self,
        guild_id: GuildId,
    ) -> impl std::future::Future<Output = Result<Option<ActiveMessage>, ActiveMessageRepoError>> + Send
    {
        let g = self.inner.lock().expect("FakeActiveMessage poisoned");
        std::future::ready(Ok(g.get(&guild_id).cloned()))
    }

    fn upsert(
        &self,
        record: ActiveMessage,
    ) -> impl std::future::Future<Output = Result<(), ActiveMessageRepoError>> + Send {
        let mut g = self.inner.lock().expect("FakeActiveMessage poisoned");
        g.insert(record.guild_id, record);
        std::future::ready(Ok(()))
    }

    fn delete(
        &self,
        guild_id: GuildId,
    ) -> impl std::future::Future<Output = Result<(), ActiveMessageRepoError>> + Send {
        let mut g = self.inner.lock().expect("FakeActiveMessage poisoned");
        g.remove(&guild_id);
        std::future::ready(Ok(()))
    }
}
