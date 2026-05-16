use std::sync::Arc;

use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{Event, EventTypeFlags, Shard, StreamExt};
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue,
};
use twilight_model::application::interaction::InteractionData;

use core_infra::SeaTaskRepository;
use core_lib::domain::{CategoryId, ChannelId, GuildId, RoleId, TaskState, ThreadId, UserId};
use core_lib::ports::{GuildSettingsUpdate, TaskRepository};
use discord_task::commands::{ConfigSetInput, CountInput, TaskStateInput};
use discord_task::events::GatewayEventHandler;

use crate::di::{AppState, ConcreteAnyEventHandler, ConcreteCommandHandlers};

pub async fn run_gateway_loop(
    shard: &mut Shard,
    handlers: &[ConcreteAnyEventHandler],
    cmd_handlers: &ConcreteCommandHandlers,
    state: &AppState,
    cache: &InMemoryCache,
) {
    loop {
        let event = match shard.next_event(EventTypeFlags::all()).await {
            Some(Ok(e)) => e,
            Some(Err(e)) => {
                tracing::warn!(%e, "shard error");
                continue;
            }
            None => break,
        };
        cache.update(&event);
        for h in handlers {
            if let Err(e) = h.handle(&event).await {
                tracing::error!(%e, "event handler error");
            }
        }
        if let Event::InteractionCreate(ref ic) = event {
            dispatch_command(ic, cmd_handlers, state).await;
        }
    }
}

async fn dispatch_command(
    ic: &twilight_model::gateway::payload::incoming::InteractionCreate,
    handlers: &ConcreteCommandHandlers,
    state: &AppState,
) {
    let Some(guild_id_raw) = ic.guild_id else {
        return;
    };
    let guild_id = GuildId(guild_id_raw.get());
    let interaction_id = ic.id.get();
    let interaction_token = ic.token.clone();

    let Some(InteractionData::ApplicationCommand(ref data)) = ic.data else {
        return;
    };

    let actor = ic
        .member
        .as_ref()
        .and_then(|m| m.user.as_ref())
        .map(|u| UserId(u.id.get()));

    match data.name.as_str() {
        "config" => {
            if let Some(input) =
                parse_config_input(guild_id, interaction_id, interaction_token, data, actor)
            {
                if let Err(e) = handlers.config_set.handle(input).await {
                    tracing::error!(%e, "config_set handler error");
                }
            } else {
                tracing::warn!("failed to parse /config command options");
            }
        }
        "task" => {
            let Some(channel_id_raw) = ic.channel.as_ref().map(|c| c.id) else {
                tracing::warn!("task command invoked outside a channel");
                return;
            };
            let thread_id = ThreadId(channel_id_raw.get());
            let task_repo = SeaTaskRepository::new(Arc::clone(&state.db));
            let task_id = match task_repo.find_by_thread(thread_id).await {
                Ok(Some(task)) => task.id,
                Ok(None) => {
                    tracing::warn!(thread_id = thread_id.0, "no task found for thread");
                    return;
                }
                Err(e) => {
                    tracing::error!(%e, "find_by_thread failed");
                    return;
                }
            };
            if let Some(next_state) = parse_task_state(&data.options) {
                let input = TaskStateInput {
                    task_id,
                    next_state,
                    interaction_id,
                    interaction_token,
                };
                if let Err(e) = handlers.task_state.handle(input).await {
                    tracing::error!(%e, "task_state handler error");
                }
            } else {
                tracing::warn!("failed to parse /task state from sub-command");
            }
        }
        "count" => {
            let pattern_name = find_string_option(&data.options, "pattern");
            let input = CountInput {
                guild_id,
                pattern_name,
                interaction_id,
                interaction_token,
            };
            if let Err(e) = handlers.count.handle(input).await {
                tracing::error!(%e, "count handler error");
            }
        }
        other => {
            tracing::warn!(command = other, "unknown slash command received");
        }
    }
}

fn parse_config_input(
    guild_id: GuildId,
    interaction_id: u64,
    token: String,
    data: &twilight_model::application::interaction::application_command::CommandData,
    actor: Option<UserId>,
) -> Option<ConfigSetInput> {
    let first = data.options.first()?;
    if first.name != "set" {
        return None;
    }

    let CommandOptionValue::SubCommand(ref sub_opts) = first.value else {
        return None;
    };

    let field = find_string_option(sub_opts, "field")?;
    let value = find_string_option(sub_opts, "value");
    let update = parse_guild_settings_update(&field, value.as_deref())?;

    Some(ConfigSetInput {
        guild_id,
        interaction_id,
        interaction_token: token,
        update,
        actor,
    })
}

fn parse_guild_settings_update(field: &str, value: Option<&str>) -> Option<GuildSettingsUpdate> {
    match field {
        "forum_channel" => {
            let id = value?.parse::<u64>().ok()?;
            Some(GuildSettingsUpdate::ForumChannel(Some(ChannelId(id))))
        }
        "active_channel" => {
            let id = value?.parse::<u64>().ok()?;
            Some(GuildSettingsUpdate::ActiveChannel(Some(ChannelId(id))))
        }
        "notify_channel" => {
            let id = value?.parse::<u64>().ok()?;
            Some(GuildSettingsUpdate::NotifyChannel(Some(ChannelId(id))))
        }
        "watch_category" => {
            let id = value?.parse::<u64>().ok()?;
            Some(GuildSettingsUpdate::WatchCategory(Some(CategoryId(id))))
        }
        "user_role" => {
            let id = value?.parse::<u64>().ok()?;
            Some(GuildSettingsUpdate::UserRole(Some(RoleId(id))))
        }
        "locale" => Some(GuildSettingsUpdate::Locale(value.map(str::to_string))),
        _ => None,
    }
}

fn parse_task_state(options: &[CommandDataOption]) -> Option<TaskState> {
    let first = options.first()?;
    match first.name.as_str() {
        "done" => Some(TaskState::Done),
        "cancel" => Some(TaskState::Cancelled),
        "start" => Some(TaskState::InProgress),
        _ => None,
    }
}

fn find_string_option(options: &[CommandDataOption], name: &str) -> Option<String> {
    options.iter().find_map(|opt| {
        if opt.name == name {
            if let CommandOptionValue::String(ref s) = opt.value {
                return Some(s.clone());
            }
        }
        None
    })
}
