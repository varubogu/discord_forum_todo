//! Database migrations. Actual migration files land in Phase 4.

pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Filled in Phase 4 — e.g. Box::new(m20260514_000001_init::Migration),
        Vec::new()
    }
}
