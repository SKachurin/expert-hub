use rust_decimal::Decimal;
use sea_orm::{DatabaseConnection, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::services::{
    calendar_connections::{
        upsert_calendar_connection, UpsertCalendarConnectionData,
    },
    experts::{
        upsert_expert, UpsertExpertData,
    },
};

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
    pub calendar_link: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterExpertSetupResponse {
    pub expert_id: i64,
    pub calendar_id: Option<i64>,
    pub created: bool,
}

pub async fn register_expert_setup(
    db: &DatabaseConnection,
    req: RegisterExpertSetupRequest,
) -> Result<RegisterExpertSetupResponse, String> {
    let tx = db
        .begin()
        .await
        .map_err(|e| format!("failed to start transaction: {e}"))?;

    let calendar_id = if req.calendar_provider.as_deref().unwrap_or("").trim().is_empty() {
        None
    } else {
        let calendar = upsert_calendar_connection(
            &tx,
            UpsertCalendarConnectionData {
                provider: req.calendar_provider.clone().unwrap(),
                link: req.calendar_link.clone(),
            },
        )
        .await?;

        Some(calendar.id)
    };

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
            calendar_id,
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

    tx.commit()
        .await
        .map_err(|e| format!("failed to commit transaction: {e}"))?;

    Ok(RegisterExpertSetupResponse {
        expert_id: expert.id,
        calendar_id,
        created: expert.created,
    })
}