use chrono::Utc;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Set};
use serde::Serialize;

use crate::entities::calendar_connections;

#[derive(Debug)]
pub struct UpsertCalendarConnectionData {
    pub expert_id: i64,
    pub provider: String,
    pub connection_label: Option<String>,
    pub is_primary: bool,
    pub is_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct UpsertCalendarConnectionResponse {
    pub id: i64,
    pub expert_id: i64,
    pub provider: String,
    pub connection_label: Option<String>,
    pub is_primary: bool,
    pub is_enabled: bool,
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

    let active = calendar_connections::ActiveModel {
        expert_id: Set(data.expert_id),
        provider: Set(normalize_provider(&data.provider)?),
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