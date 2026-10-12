use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    // `cli::run_cli` in sea-orm-migration 2.0 installs the global tracing
    // subscriber. Initializing another one here panics at startup.
    cli::run_cli(migration::Migrator).await;
}
