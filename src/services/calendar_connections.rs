use chrono::Utc;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Set};
use serde::Serialize;

use crate::entities::calendar_connections;

#[derive(Debug)]
pub struct UpsertCalendarConnectionData {
    pub provider: String,
    pub link: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpsertCalendarConnectionResponse {
    pub id: i64,
    pub provider: String,
    pub link: Option<String>,
}

pub async fn upsert_calendar_connection<C>(
    db: &C,
    data: UpsertCalendarConnectionData,
) -> Result<UpsertCalendarConnectionResponse, String>
where
    C: ConnectionTrait,
{
    if data.provider.trim().is_empty() {
        return Err("calendar_provider is required".to_string());
    }

    let now = Utc::now().fixed_offset();

    let active = calendar_connections::ActiveModel {
        provider: Set(data.provider.clone()),
        link: Set(data.link.clone()),
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
        provider: model.provider,
        link: model.link,
    })
}