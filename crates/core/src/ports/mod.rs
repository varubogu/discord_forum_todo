//! Port traits — abstract interfaces over side effects. Implementations live
//! in adapter crates (`core_infra` and `discord_task_infra`).

pub mod active_message_repository;
pub mod channel_pattern_repository;
pub mod clock;
pub mod guild_settings_repository;
pub mod project_repository;
pub mod task_repository;

pub use active_message_repository::{ActiveMessageRepoError, ActiveMessageRepository};
pub use channel_pattern_repository::{ChannelPatternRepoError, ChannelPatternRepository};
pub use clock::{Clock, FakeClock};
pub use guild_settings_repository::{
    GuildSettingsRepoError, GuildSettingsRepository, GuildSettingsUpdate,
};
pub use project_repository::{ProjectRepoError, ProjectRepository};
pub use task_repository::{StaleQuery, StaleTaskRow, TaskRepoError, TaskRepository};
