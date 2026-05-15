//! In-memory fakes for the ports declared in `core::ports`. Enabled internally
//! for the crate's own tests and externally via the `test-support` feature.

pub mod fake_active_message_repository;
pub mod fake_channel_pattern_repository;
pub mod fake_guild_settings_repository;
pub mod fake_project_repository;
pub mod fake_task_repository;
pub mod fake_translator;

pub use fake_active_message_repository::FakeActiveMessageRepository;
pub use fake_channel_pattern_repository::FakeChannelPatternRepository;
pub use fake_guild_settings_repository::FakeGuildSettingsRepository;
pub use fake_project_repository::FakeProjectRepository;
pub use fake_task_repository::FakeTaskRepository;
pub use fake_translator::FakeTranslator;
