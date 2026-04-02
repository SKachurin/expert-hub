use crate::config::AppConfig;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoogleCalendarCandidate {
    pub id: String,
    pub summary: String,
    pub time_zone: Option<String>,
    pub access_role: Option<String>,
    pub primary: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoogleOAuthSession {
    pub session_id: String,
    pub account_email: Option<String>,
    pub provider_account_id: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub scopes: Option<String>,
    pub calendars: Vec<GoogleCalendarCandidate>,
    pub selected_calendar_ids: Vec<String>,
    pub created_at_unix: i64,
}

pub struct AppState {
    pub config: AppConfig,
    pub db: DatabaseConnection,
    pub google_oauth_sessions: Mutex<HashMap<String, GoogleOAuthSession>>,
}

impl AppState {
    pub fn new(config: AppConfig, db: DatabaseConnection) -> Self {
        Self {
            config,
            db,
            google_oauth_sessions: Mutex::new(HashMap::new()),
        }
    }
}