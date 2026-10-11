use std::sync::Mutex;

use crate::domain::{ChannelPattern, ChannelPatternId, GuildId, NewChannelPattern};
use crate::ports::{ChannelPatternRepoError, ChannelPatternRepository};

#[derive(Default)]
pub struct FakeChannelPatternRepository {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    next_id: i64,
    rows: Vec<ChannelPattern>,
}

impl FakeChannelPatternRepository {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ChannelPatternRepository for FakeChannelPatternRepository {
    fn list(
        &self,
        guild_id: GuildId,
    ) -> impl std::future::Future<Output = Result<Vec<ChannelPattern>, ChannelPatternRepoError>> + Send
    {
        let g = self.inner.lock().expect("FakeChannelPattern poisoned");
        std::future::ready(Ok(g
            .rows
            .iter()
            .filter(|p| p.guild_id == guild_id)
            .cloned()
            .collect()))
    }

    fn insert(
        &self,
        pattern: NewChannelPattern,
    ) -> impl std::future::Future<Output = Result<ChannelPattern, ChannelPatternRepoError>> + Send
    {
        let mut g = self.inner.lock().expect("FakeChannelPattern poisoned");
        if g.rows
            .iter()
            .any(|p| p.guild_id == pattern.guild_id && p.name == pattern.name)
        {
            return std::future::ready(Err(ChannelPatternRepoError::DuplicateName(pattern.name)));
        }
        g.next_id += 1;
        let persisted = ChannelPattern {
            id: ChannelPatternId(g.next_id),
            guild_id: pattern.guild_id,
            name: pattern.name,
            kind: pattern.kind,
            pattern: pattern.pattern,
            created_at: pattern.created_at,
        };
        g.rows.push(persisted.clone());
        std::future::ready(Ok(persisted))
    }

    fn delete_by_name(
        &self,
        guild_id: GuildId,
        name: &str,
    ) -> impl std::future::Future<Output = Result<(), ChannelPatternRepoError>> + Send {
        let mut g = self.inner.lock().expect("FakeChannelPattern poisoned");
        let before = g.rows.len();
        g.rows
            .retain(|p| !(p.guild_id == guild_id && p.name == name));
        if g.rows.len() == before {
            return std::future::ready(Err(ChannelPatternRepoError::NotFound));
        }
        std::future::ready(Ok(()))
    }

    fn find_by_id(
        &self,
        id: ChannelPatternId,
    ) -> impl std::future::Future<Output = Result<Option<ChannelPattern>, ChannelPatternRepoError>> + Send
    {
        let g = self.inner.lock().expect("FakeChannelPattern poisoned");
        std::future::ready(Ok(g.rows.iter().find(|p| p.id == id).cloned()))
    }
}
