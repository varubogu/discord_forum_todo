pub mod sea_active_message_repository;
pub mod sea_channel_pattern_repository;
pub mod sea_guild_settings_repository;
pub mod sea_project_repository;
pub mod sea_task_repository;

pub use sea_active_message_repository::SeaActiveMessageRepository;
pub use sea_channel_pattern_repository::SeaChannelPatternRepository;
pub use sea_guild_settings_repository::SeaGuildSettingsRepository;
pub use sea_project_repository::SeaProjectRepository;
pub use sea_task_repository::SeaTaskRepository;

// Discord snowflakes fit in 63 bits so sign-bit wrapping / sign loss cannot occur.
#[allow(clippy::cast_possible_wrap)]
pub(super) fn to_db_id(id: u64) -> i64 {
    id as i64
}

#[allow(clippy::cast_sign_loss)]
pub(super) fn from_db_id(id: i64) -> u64 {
    id as u64
}
