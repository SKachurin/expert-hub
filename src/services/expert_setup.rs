use rust_decimal::Decimal;
use sea_orm::{DatabaseConnection, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
    pub calendar_provider: Option<String>,

    #[serde(default)]
    pub google_session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterExpertSetupResponse {
    pub expert_id: i64,
    pub calendar_connection_ids: Vec<i64>,
    pub created: bool,
}

pub async fn register_expert_setup(
    state: &AppState,
    req: RegisterExpertSetupRequest,
) -> Result<RegisterExpertSetupResponse, String> {
    let tx = state
        .db
        .begin()
        .await
        .map_err(|e| format!("failed to start transaction: {e}"))?;

    let expert = upsert_expert(
        &tx,
        UpsertExpertData {
            telegram_id: req.telegram_id,
            first_name: req.first_name,
            last_name: req.last_name,
            username: req.username,
            photo_url: req.photo_url,
            display_name: req.display_name,
            telegram_bio: req.telegram_bio,
            ton_wallet_address: req.ton_wallet_address,
            timezone: req.timezone,
            hourly_rate: req.hourly_rate,
            currency: req.currency,
            working_days: req.working_days,
            work_start_time: req.work_start_time,
            work_end_time: req.work_end_time,
            allowed_session_durations: req.allowed_session_durations,
        },
    )
    .await?;

    let mut calendar_connection_ids = Vec::new();

    if req.calendar_provider.as_deref() == Some("google") {
        let session_id = req
            .google_session_id
            .as_deref()
            .ok_or_else(|| "google_session_id is required for google provider".to_string())?;

        let session = {
            let sessions = state
                .google_oauth_sessions
                .lock()
                .map_err(|_| "failed to read google oauth sessions".to_string())?;

            sessions
                .get(session_id)
                .cloned()
                .ok_or_else(|| "google session not found".to_string())?
        };

        if session.selected_calendar_ids.is_empty() {
            return Err("no google calendars selected".to_string());
        }

        for calendar_id in &session.selected_calendar_ids {
            let selected = session
                .calendars
                .iter()
                .find(|v| &v.id == calendar_id)
                .ok_or_else(|| format!("selected google calendar not found: {}", calendar_id))?;

            let now = chrono::Utc::now().fixed_offset();

            let active = crate::entities::calendar_connections::ActiveModel {
                expert_id: sea_orm::Set(expert.id),
                provider: sea_orm::Set("google".to_string()),
                connection_label: sea_orm::Set(Some(selected.summary.clone())),
                is_primary: sea_orm::Set(calendar_connection_ids.is_empty()),
                is_enabled: sea_orm::Set(true),
                connection_status: sea_orm::Set("connected".to_string()),
                account_email: sea_orm::Set(session.account_email.clone()),
                provider_account_id: sea_orm::Set(session.provider_account_id.clone()),
                provider_user_uri: sea_orm::Set(None),
                provider_organization_uri: sea_orm::Set(None),
                selected_calendar_id: sea_orm::Set(Some(selected.id.clone())),
                selected_calendar_name: sea_orm::Set(Some(selected.summary.clone())),
                selected_calendar_timezone: sea_orm::Set(selected.time_zone.clone()),
                selected_event_type_uri: sea_orm::Set(None),
                selected_event_type_name: sea_orm::Set(None),
                selected_scheduling_url: sea_orm::Set(None),
                access_token: sea_orm::Set(Some(session.access_token.clone())),
                refresh_token: sea_orm::Set(session.refresh_token.clone()),
                token_expires_at: sea_orm::Set(None),
                scopes_json: sea_orm::Set(session.scopes.as_ref().map(|v| {
                    json!(v.split_whitespace().collect::<Vec<_>>())
                })),
                provider_metadata: sea_orm::Set(Some(json!({
                    "primary": selected.primary,
                    "access_role": selected.access_role,
                    "google_session_id": session.session_id,
                }))),
                sync_cursor: sea_orm::Set(None),
                last_sync_at: sea_orm::Set(None),
                last_sync_status: sea_orm::Set(None),
                last_sync_error: sea_orm::Set(None),
                webhook_signing_secret: sea_orm::Set(None),
                public_link: sea_orm::Set(None),
                created_at: sea_orm::Set(now),
                updated_at: sea_orm::Set(now),
                ..Default::default()
            };

            let model = sea_orm::ActiveModelTrait::insert(active, &tx)
                .await
                .map_err(|e| format!("failed to insert google calendar connection: {e}"))?;

            calendar_connection_ids.push(model.id);
        }

        if let Ok(mut sessions) = state.google_oauth_sessions.lock() {
            sessions.remove(session_id);
        }
    } else if let Some(provider) = req.calendar_provider {
        let calendar = upsert_calendar_connection(
            &tx,
            UpsertCalendarConnectionData {
                expert_id: expert.id,
                provider,
                connection_label: None,
                is_primary: true,
                is_enabled: true,
            },
        )
        .await?;

        calendar_connection_ids.push(calendar.id);
    }

    tx.commit()
        .await
        .map_err(|e| format!("failed to commit transaction: {e}"))?;

    Ok(RegisterExpertSetupResponse {
        expert_id: expert.id,
        calendar_connection_ids,
        created: expert.created,
    })
}