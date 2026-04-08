use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set, DeleteResult};
use serde::Serialize;
use serde_json::Value;

use crate::entities::calendar_connections;

#[derive(Debug)]
pub struct UpsertCalendarConnectionData {
    pub expert_id: i64,
    pub provider: String,
    pub connection_label: Option<String>,
    pub is_primary: bool,
    pub is_enabled: bool,

    pub connection_status: Option<String>,
    pub account_email: Option<String>,
    pub provider_account_id: Option<String>,
    pub provider_user_uri: Option<String>,
    pub provider_organization_uri: Option<String>,
    pub selected_calendar_id: Option<String>,
    pub selected_calendar_name: Option<String>,
    pub selected_calendar_timezone: Option<String>,
    pub selected_event_type_uri: Option<String>,
    pub selected_event_type_name: Option<String>,
    pub selected_scheduling_url: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<FixedOffset>>,
    pub scopes_json: Option<Value>,
    pub provider_metadata: Option<Value>,
    pub sync_cursor: Option<String>,
    pub last_sync_at: Option<DateTime<FixedOffset>>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
    pub webhook_signing_secret: Option<String>,
    pub public_link: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpsertCalendarConnectionResponse {
    pub id: i64,
    pub expert_id: i64,
    pub provider: String,
    pub connection_label: Option<String>,
    pub is_primary: bool,
    pub is_enabled: bool,
    pub connection_status: String,
    pub selected_calendar_id: Option<String>,
    pub selected_calendar_name: Option<String>,
}

pub async fn upsert_calendar_connection<C>(
    db: &C,
    data: UpsertCalendarConnectionData,
) -> Result<UpsertCalendarConnectionResponse, String>
where
    C: ConnectionTrait,
{
    validate_data(&data)?;

    let now = Utc::now().fixed_offset();

    let provider = normalize_provider(&data.provider)?;
    let connection_status = data
        .connection_status
        .clone()
        .unwrap_or_else(|| "pending".to_string());

    let active = calendar_connections::ActiveModel {
        expert_id: Set(data.expert_id),
        provider: Set(provider),
        connection_label: Set(
            data.connection_label
                .clone()
                .and_then(|v| {
                    let trimmed = v.trim().to_string();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed)
                    }
                }),
        ),
        is_primary: Set(data.is_primary),
        is_enabled: Set(data.is_enabled),

        connection_status: Set(connection_status),
        account_email: Set(data.account_email.clone()),
        provider_account_id: Set(data.provider_account_id.clone()),
        provider_user_uri: Set(data.provider_user_uri.clone()),
        provider_organization_uri: Set(data.provider_organization_uri.clone()),
        selected_calendar_id: Set(data.selected_calendar_id.clone()),
        selected_calendar_name: Set(data.selected_calendar_name.clone()),
        selected_calendar_timezone: Set(data.selected_calendar_timezone.clone()),
        selected_event_type_uri: Set(data.selected_event_type_uri.clone()),
        selected_event_type_name: Set(data.selected_event_type_name.clone()),
        selected_scheduling_url: Set(data.selected_scheduling_url.clone()),
        access_token: Set(data.access_token.clone()),
        refresh_token: Set(data.refresh_token.clone()),
        token_expires_at: Set(data.token_expires_at),
        scopes_json: Set(data.scopes_json.clone()),
        provider_metadata: Set(data.provider_metadata.clone()),
        sync_cursor: Set(data.sync_cursor.clone()),
        last_sync_at: Set(data.last_sync_at),
        last_sync_status: Set(data.last_sync_status.clone()),
        last_sync_error: Set(data.last_sync_error.clone()),
        webhook_signing_secret: Set(data.webhook_signing_secret.clone()),
        public_link: Set(data.public_link.clone()),

        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let model = active
        .insert(db)
        .await
        .map_err(|e| format!("failed to insert calendar connection: {e}"))?;

    Ok(UpsertCalendarConnectionResponse {
        id: model.id,
        expert_id: model.expert_id,
        provider: model.provider,
        connection_label: model.connection_label,
        is_primary: model.is_primary,
        is_enabled: model.is_enabled,
        connection_status: model.connection_status,
        selected_calendar_id: model.selected_calendar_id,
        selected_calendar_name: model.selected_calendar_name,
    })
}

fn validate_data(data: &UpsertCalendarConnectionData) -> Result<(), String> {
    if data.expert_id <= 0 {
        return Err("expert_id is required".to_string());
    }

    if data.provider.trim().is_empty() {
        return Err("provider is required".to_string());
    }

    normalize_provider(&data.provider).map(|_| ())
}

fn normalize_provider(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_lowercase();

    match normalized.as_str() {
        "google" | "google_calendar" => Ok("google".to_string()),
        "calendly" => Ok("calendly".to_string()),
        _ => Err(format!("unsupported calendar provider: {value}")),
    }
}

pub async fn delete_calendar_connection_for_expert<C>(
    db: &C,
    expert_id: i64,
    connection_id: i64,
) -> Result<(), String>
where
    C: ConnectionTrait,
{
    let existing = calendar_connections::Entity::find_by_id(connection_id)
        .one(db)
        .await
        .map_err(|e| format!("failed to load calendar connection: {e}"))?
        .ok_or_else(|| "calendar connection not found".to_string())?;

    if existing.expert_id != expert_id {
        return Err("calendar connection does not belong to this expert".to_string());
    }

    calendar_connections::Entity::delete_by_id(connection_id)
        .exec(db)
        .await
        .map_err(|e| format!("failed to delete calendar connection: {e}"))?;

    Ok(())
}

pub async fn update_google_connection_tokens<C>(
    db: &C,
    connection_id: i64,
    access_token: String,
    token_expires_at: Option<DateTime<Utc>>,
) -> Result<(), String>
where
    C: ConnectionTrait,
{
    let model = calendar_connections::Entity::find_by_id(connection_id)
        .one(db)
        .await
        .map_err(|e| format!("failed to load calendar connection: {e}"))?
        .ok_or_else(|| "calendar connection not found".to_string())?;

    let mut active: calendar_connections::ActiveModel = model.into();

    active.access_token = Set(Some(access_token));
    active.token_expires_at = Set(token_expires_at.map(|v| v.fixed_offset()));
    active.updated_at = Set(Utc::now().fixed_offset());

    active
        .update(db)
        .await
        .map_err(|e| format!("failed to update calendar connection tokens: {e}"))?;

    Ok(())
}