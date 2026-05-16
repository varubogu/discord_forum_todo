//! Environment-driven configuration loaded once at startup.

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    pub discord_token: String,
    pub database_url: String,
    #[serde(default = "default_scheduler_interval_secs")]
    pub scheduler_interval_secs: u64,
    #[serde(default = "default_scheduler_batch_size")]
    pub scheduler_batch_size: u32,
    #[serde(default = "default_shutdown_timeout_secs")]
    pub shutdown_timeout_secs: u64,
    #[serde(default = "default_locale")]
    pub default_locale: String,
    #[serde(default = "default_locales_dir")]
    pub locales_dir: String,
}

fn default_scheduler_interval_secs() -> u64 {
    60
}

fn default_scheduler_batch_size() -> u32 {
    200
}

fn default_shutdown_timeout_secs() -> u64 {
    30
}

fn default_locale() -> String {
    "en-US".to_string()
}

fn default_locales_dir() -> String {
    "./locales".to_string()
}

impl BotConfig {
    pub fn from_env() -> Result<Self> {
        use figment::providers::Env;
        use figment::Figment;

        Figment::new()
            .merge(Env::prefixed("BOT_"))
            .merge(Env::raw().only(&["DISCORD_TOKEN", "DATABASE_URL"]))
            .extract::<RawConfig>()
            .context("failed to read environment configuration")
            .map(Into::into)
    }

    pub fn database_url_redacted(&self) -> String {
        match url_strip_password(&self.database_url) {
            Some(s) => s,
            None => "<unparseable>".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    discord_token: String,
    database_url: String,
    #[serde(default = "default_scheduler_interval_secs")]
    scheduler_interval_secs: u64,
    #[serde(default = "default_scheduler_batch_size")]
    scheduler_batch_size: u32,
    #[serde(default = "default_shutdown_timeout_secs")]
    shutdown_timeout_secs: u64,
    #[serde(default = "default_locale")]
    default_locale: String,
    #[serde(default = "default_locales_dir")]
    locales_dir: String,
}

impl From<RawConfig> for BotConfig {
    fn from(r: RawConfig) -> Self {
        Self {
            discord_token: r.discord_token,
            database_url: r.database_url,
            scheduler_interval_secs: r.scheduler_interval_secs,
            scheduler_batch_size: r.scheduler_batch_size,
            shutdown_timeout_secs: r.shutdown_timeout_secs,
            default_locale: r.default_locale,
            locales_dir: r.locales_dir,
        }
    }
}

fn url_strip_password(s: &str) -> Option<String> {
    let scheme_end = s.find("://")?;
    let after_scheme = &s[scheme_end + 3..];
    let (auth, rest) = match after_scheme.find('@') {
        Some(at) => (&after_scheme[..at], &after_scheme[at + 1..]),
        None => return Some(s.to_string()),
    };
    let user = auth.split_once(':').map_or(auth, |(u, _)| u);
    Some(format!("{}://{user}@{rest}", &s[..scheme_end]))
}
