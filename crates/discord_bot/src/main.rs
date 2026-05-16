use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use migration::{Migrator, MigratorTrait};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Config, Intents, Shard, ShardId};

mod config;
mod di;
mod gateway;
mod scheduler;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cfg = config::BotConfig::from_env()?;
    tracing::info!(
        database_url = %cfg.database_url_redacted(),
        scheduler_interval_secs = cfg.scheduler_interval_secs,
        scheduler_batch_size = cfg.scheduler_batch_size,
        default_locale = %cfg.default_locale,
        "starting discord_forum_todo",
    );

    let db = Arc::new(sea_orm::Database::connect(&cfg.database_url).await?);
    Migrator::up(&*db, None).await?;

    let http = Arc::new(twilight_http::Client::new(cfg.discord_token.clone()));
    let application_id = http.current_user_application().await?.model().await?.id;

    let locales_dir = std::path::Path::new(&cfg.locales_dir);
    let state = Arc::new(di::build_app_state(
        Arc::clone(&db),
        Arc::clone(&http),
        application_id,
        locales_dir,
    ));

    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::CHANNEL | ResourceType::GUILD | ResourceType::MESSAGE)
        .build();

    let intents = Intents::GUILDS | Intents::GUILD_MESSAGES;
    let shard_config = Config::new(cfg.discord_token.clone(), intents);
    let mut shard = Shard::with_config(ShardId::ONE, shard_config);

    let handlers = di::build_event_handlers(&state);
    let cmd_handlers = di::build_command_handlers(&state);

    let interval = cfg.scheduler_interval_secs;
    let batch = cfg.scheduler_batch_size;
    let scheduler_state = Arc::clone(&state);
    let sched = tokio::spawn(async move {
        scheduler::run_scheduler(scheduler_state, interval, batch).await;
    });

    tokio::select! {
        () = gateway::run_gateway_loop(&mut shard, &handlers, &cmd_handlers, &state, &cache) => {},
        () = async { tokio::signal::ctrl_c().await.ok(); } => {
            tracing::info!("shutdown signal received");
        }
    }

    shard.close(twilight_gateway::CloseFrame::NORMAL);

    let timeout = Duration::from_secs(cfg.shutdown_timeout_secs);
    tokio::time::timeout(timeout, async { sched.abort() })
        .await
        .ok();

    Ok(())
}
