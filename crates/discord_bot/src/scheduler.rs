use std::sync::Arc;
use std::time::Duration;

use core_infra::entities::guild_settings::{Column, Entity as GsEntity};
use core_infra::SeaTaskRepository;
use core_lib::domain::{ChannelId, GuildId};
use core_lib::i18n::Locale;
use core_lib::ports::{Clock, TaskRepository};
use core_lib::usecases::{render_active_tasks, DetectStaleTasks};
use discord_task::ports::{ActiveMessageService, TaskNotifier};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::di::AppState;

pub async fn run_scheduler(state: Arc<AppState>, interval_secs: u64, batch_size: u32) {
    let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
    loop {
        ticker.tick().await;
        run_stale_notification(&state, batch_size).await;
        run_active_message_sync(&state).await;
    }
}

async fn run_stale_notification(state: &AppState, batch_size: u32) {
    let task_repo = SeaTaskRepository::new(Arc::clone(&state.db));
    let detect = DetectStaleTasks::new(SeaTaskRepository::new(Arc::clone(&state.db)), state.clock);

    let rows = match detect.execute(batch_size, None).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(%e, "stale task detection failed");
            return;
        }
    };

    let mut notified_ids = Vec::new();
    for row in &rows {
        let locale = row
            .locale
            .as_deref()
            .and_then(|s| s.parse::<unic_langid::LanguageIdentifier>().ok())
            .map_or_else(Locale::fallback, Locale);

        match state
            .notifier
            .notify_stale(row.notify_channel_id, row.user_role_id, &row.task, &locale)
            .await
        {
            Ok(()) => notified_ids.push(row.task.id),
            Err(e) => tracing::error!(%e, task_id = row.task.id.0, "notify_stale failed"),
        }
    }

    if !notified_ids.is_empty() {
        if let Err(e) = task_repo
            .mark_notified(&notified_ids, state.clock.now())
            .await
        {
            tracing::error!(%e, "mark_notified failed");
        }
    }
}

async fn run_active_message_sync(state: &AppState) {
    let guilds = match GsEntity::find()
        .filter(Column::ActiveChannelId.is_not_null())
        .all(&*state.db)
        .await
    {
        Ok(g) => g,
        Err(e) => {
            tracing::error!(%e, "failed to query guild settings for active message sync");
            return;
        }
    };

    for gs in guilds {
        let Some(active_channel_raw) = gs.active_channel_id else {
            continue;
        };

        #[allow(clippy::cast_sign_loss)]
        let guild_id = GuildId(gs.guild_id as u64);
        #[allow(clippy::cast_sign_loss)]
        let active_channel_id = ChannelId(active_channel_raw as u64);

        let task_repo = SeaTaskRepository::new(Arc::clone(&state.db));
        let tasks = match task_repo.fetch_active(guild_id).await {
            Ok(t) => t,
            Err(e) => {
                tracing::error!(%e, guild_id = guild_id.0, "fetch_active failed");
                continue;
            }
        };

        let locale = gs
            .locale
            .as_deref()
            .and_then(|s| s.parse::<unic_langid::LanguageIdentifier>().ok())
            .map_or_else(Locale::fallback, Locale);

        let now = state.clock.now();
        let content = render_active_tasks(&tasks, &locale, &*state.translator, now);
        let hash = content.hash(&locale);

        if let Err(e) = state
            .active_message_service
            .sync_active_message(guild_id, active_channel_id, content.pages, hash.to_vec())
            .await
        {
            tracing::error!(%e, guild_id = guild_id.0, "active message sync failed");
        }
    }
}
