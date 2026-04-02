use migration::Migrator;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Database;
use std::env;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    Migrator::up(&db, None)
        .await
        .expect("Failed to apply migrations");

    println!("Migrations applied successfully");
}