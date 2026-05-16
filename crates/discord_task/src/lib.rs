//! `discord_forum_todo_discord_task` — Twilight-based adapter layer.
//!
//! Provides:
//! - [`ports`] — Discord-specific port traits (`ForumService`, `ChannelInspector`,
//!   `ActiveMessageService`, `TaskNotifier`, `InteractionResponder`)
//! - [`commands`] — Slash-command handler structs (`ConfigSetHandler`,
//!   `TaskStateHandler`, `CountHandler`)
//! - [`events`] — Gateway event handler structs + [`events::GatewayEventHandler`]
//!   trait + [`events::AnyEventHandler`] dispatch enum

pub mod commands;
pub mod events;
pub mod ports;
