pub use sea_orm_migration::prelude::*;

mod m20260319_000001_create_calendar_connections_table;
mod m20260319_000002_create_experts_table;
mod m20260319_000003_create_tags_table;
mod m20260319_000004_create_categories_table;
mod m20260319_000005_create_reviews_table;
mod m20260319_000006_create_bookings_table;
mod m20260319_000007_create_expert_tags_table;
mod m20260319_000008_create_expert_categories_table;
mod m20260320_000009_create_payments_table;
mod m20260320_000010_create_telegram_call_events_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260319_000001_create_calendar_connections_table::Migration),
            Box::new(m20260319_000002_create_experts_table::Migration),
            Box::new(m20260319_000003_create_tags_table::Migration),
            Box::new(m20260319_000004_create_categories_table::Migration),
            Box::new(m20260319_000005_create_reviews_table::Migration),
            Box::new(m20260319_000006_create_bookings_table::Migration),
            Box::new(m20260319_000007_create_expert_tags_table::Migration),
            Box::new(m20260319_000008_create_expert_categories_table::Migration),
            Box::new(m20260320_000009_create_payments_table::Migration),
            Box::new(m20260320_000010_create_telegram_call_events_table::Migration),
        ]
    }
}