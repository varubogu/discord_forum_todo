//! `discord_forum_todo` — bot binary.
//!
//! Phase 2 skeleton: validates that the workspace compiles end-to-end. The real
//! bootstrap (Twilight gateway, dependency injection, scheduler, graceful
//! shutdown) lands in Phase 7.

mod config;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cfg = config::BotConfig::from_env()?;
    tracing::info!(
        database_url = %cfg.database_url_redacted(),
        discord_token_set = !cfg.discord_token.is_empty(),
        scheduler_interval_secs = cfg.scheduler_interval_secs,
        scheduler_batch_size = cfg.scheduler_batch_size,
        shutdown_timeout_secs = cfg.shutdown_timeout_secs,
        default_locale = %cfg.default_locale,
        "starting discord_forum_todo (skeleton — Phase 7 will replace this with the real bootstrap)"
    );
    Ok(())
}
