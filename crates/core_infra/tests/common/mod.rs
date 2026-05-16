use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::postgres::Postgres;

pub async fn setup_db() -> (DatabaseConnection, impl Drop) {
    let container = Postgres::default()
        .with_env_var("POSTGRES_DB", "testdb")
        .start()
        .await
        .expect("failed to start postgres container");

    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("failed to get host port");

    let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/testdb");

    let db = sea_orm::Database::connect(&url)
        .await
        .expect("failed to connect to test db");

    Migrator::up(&db, None).await.expect("migration failed");

    (db, container)
}

pub fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}
