use rust_decimal::Decimal;
use sea_orm::TransactionTrait;
use serde_json::Value;
use serde::{Deserialize, Serialize};

use crate::services::{
    calendar_connections::{
        upsert_calendar_connection, UpsertCalendarConnectionData,
    },
    experts::{
        upsert_expert, UpsertExpertData,
    },
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RegisterCalendarConnectionRequest {
    pub provider: String,

    #[serde(default)]
    pub google_session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterExpertSetupRequest {
    pub telegram_id: i64,

    #[serde(default)]
    pub first_name: String,

    #[serde(default)]
    pub last_name: String,

    #[serde(default)]
    pub username: String,

    #[serde(default)]
    pub photo_url: Option<String>,

    pub display_name: String,

    #[serde(default)]
    pub telegram_bio: Option<String>,

    pub ton_wallet_address: String,
    pub timezone: String,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub working_days: Value,
    pub work_start_time: String,
    pub work_end_time: String,
    pub allowed_session_durations: Value,

    #[serde(default)]
    pub calendar_connections: Vec<RegisterCalendarConnectionRequest>,
}

#[derive(Debug, Serialize)]
pub struct RegisterExpertSetupResponse {
    pub expert_id: i64,
    pub created: bool,
    pub calendar_connection_ids: Vec<i64>,
}

pub async fn register_expert_setup(
    state: &AppState,
    body: RegisterExpertSetupRequest,
) -> Result<RegisterExpertSetupResponse, String> {
    if body.calendar_connections.is_empty() {
        return Err("at least one calendar connection is required".to_string());
    }

    let tx = state
        .db
        .begin()
        .await
        .map_err(|e| format!("failed to start transaction: {e}"))?;

    let expert_saved = upsert_expert(
        &tx,
        UpsertExpertData {
            telegram_id: body.telegram_id,
            first_name: body.first_name.clone(),
            last_name: body.last_name.trim().to_string(),
            username: body.username.trim().to_string(),
            display_name: body.display_name.trim().to_string(),
            telegram_bio: body
                .telegram_bio
                .clone()
                .and_then(|v| {
                    let trimmed = v.trim().to_string();
                    if trimmed.is_empty() { None } else { Some(trimmed) }
                }),
            photo_url: body.photo_url.clone(),
            ton_wallet_address: body.ton_wallet_address.trim().to_string(),
            timezone: body.timezone.trim().to_string(),
            hourly_rate: body.hourly_rate,
            currency: body.currency.trim().to_uppercase(),
            working_days: body.working_days.clone(),
            work_start_time: body.work_start_time.clone(),
            work_end_time: body.work_end_time.clone(),
            allowed_session_durations: body.allowed_session_durations.clone(),
        },
    )
    .await
    .map_err(|e| format!("failed to save expert: {e}"))?;

    let mut calendar_connection_ids: Vec<i64> = Vec::new();

    for connection in &body.calendar_connections {
        let provider = connection.provider.trim().to_lowercase();

        match provider.as_str() {
            "google" | "google_calendar" => {
                let google_session_id = connection
                    .google_session_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .ok_or_else(|| "google_session_id is required for google provider".to_string())?;

                let session = {
                    let guard = state
                        .google_oauth_sessions
                        .lock()
                        .map_err(|_| "failed to read google oauth sessions".to_string())?;

                    guard
                        .get(google_session_id)
                        .cloned()
                        .ok_or_else(|| format!("google session not found: {google_session_id}"))?
                };

                if session.selected_calendar_ids.is_empty() {
                    return Err(format!(
                        "no google calendars selected for session {google_session_id}"
                    ));
                }

                for selected_id in &session.selected_calendar_ids {
                    let selected_calendar = session
                        .calendars
                        .iter()
                        .find(|c| c.id == *selected_id)
                        .ok_or_else(|| format!("selected google calendar not found: {selected_id}"))?;

                    let saved = upsert_calendar_connection(
                        &tx,
                        UpsertCalendarConnectionData {
                            expert_id: expert_saved.id,
                            provider: "google".to_string(),
                            connection_label: Some(selected_calendar.summary.clone()),
                            is_primary: calendar_connection_ids.is_empty(),
                            is_enabled: true,
                        },
                    )
                    .await
                    .map_err(|e| format!("failed to save calendar connection: {e}"))?;

                    calendar_connection_ids.push(saved.id);
                }
            }

            "calendly" => {
                return Err("calendly is not implemented yet".to_string());
            }

            other => {
                return Err(format!("unsupported calendar provider: {other}"));
            }
        }
    }

    tx.commit()
        .await
        .map_err(|e| format!("failed to commit transaction: {e}"))?;

    Ok(RegisterExpertSetupResponse {
        expert_id: expert_saved.id,
        created: expert_saved.created,
        calendar_connection_ids,
    })
}