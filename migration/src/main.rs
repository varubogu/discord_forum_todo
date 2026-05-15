use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    cli::run_cli(migration::Migrator).await;
}
