use crate::config::AppConfig;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(config: AppConfig, db: DatabaseConnection) -> Self {
        Self { config, db }
    }
}