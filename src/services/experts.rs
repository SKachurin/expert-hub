use chrono::{NaiveTime, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set,
};
use serde::Serialize;
use serde_json::Value;

use crate::entities::experts;

#[derive(Debug)]
pub struct UpsertExpertData {
    pub telegram_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub photo_url: Option<String>,
    pub display_name: String,
    pub telegram_bio: Option<String>,
    pub ton_wallet_address: String,
    pub calendar_id: Option<i64>,
    pub timezone: String,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub working_days: Value,
    pub work_start_time: String,
    pub work_end_time: String,
    pub allowed_session_durations: Value,
}

#[derive(Debug, Serialize)]
pub struct UpsertExpertResponse {
    pub id: i64,
    pub telegram_id: i64,
    pub telegram_username: String,
    pub telegram_name: String,
    pub ton_wallet_address: String,
    pub calendar_id: Option<i64>,
    pub timezone: String,
    pub created: bool,
}

pub async fn upsert_expert<C>(
    db: &C,
    data: UpsertExpertData,
) -> Result<UpsertExpertResponse, String>
where
    C: ConnectionTrait,
{
    validate_data(&data)?;

    let now = Utc::now().fixed_offset();
    let telegram_name = if data.display_name.trim().is_empty() {
        build_telegram_name(&data.first_name, &data.last_name, &data.username)
    } else {
        data.display_name.trim().to_string()
    };

    let work_start_time = parse_time(&data.work_start_time)?;
    let work_end_time = parse_time(&data.work_end_time)?;

    let existing = experts::Entity::find()
        .filter(experts::Column::TelegramId.eq(data.telegram_id))
        .one(db)
        .await
        .map_err(|e| format!("failed to query expert: {e}"))?;

    let created = existing.is_none();

    let model = match existing {
        Some(existing) => {
            let mut active: experts::ActiveModel = existing.into();

            active.telegram_username = Set(data.username.clone());
            active.telegram_name = Set(telegram_name.clone());
            active.telegram_bio = Set(data.telegram_bio.clone());
            active.ton_wallet_address = Set(data.ton_wallet_address.clone());
            active.calendar_id = Set(data.calendar_id);
            active.photo_url = Set(data.photo_url.clone());
            active.hourly_rate = Set(data.hourly_rate);
            active.currency = Set(data.currency.clone());
            active.timezone = Set(data.timezone.clone());
            active.working_days = Set(data.working_days.clone());
            active.work_start_time = Set(work_start_time);
            active.work_end_time = Set(work_end_time);
            active.allowed_session_durations = Set(data.allowed_session_durations.clone());
            active.updated_at = Set(now);

            active
                .update(db)
                .await
                .map_err(|e| format!("failed to update expert: {e}"))?
        }
        None => {
            let active = experts::ActiveModel {
                telegram_id: Set(data.telegram_id),
                telegram_username: Set(data.username.clone()),
                telegram_name: Set(telegram_name.clone()),
                telegram_bio: Set(data.telegram_bio.clone()),
                ton_wallet_address: Set(data.ton_wallet_address.clone()),
                calendar_id: Set(data.calendar_id),
                photo_url: Set(data.photo_url.clone()),
                hourly_rate: Set(data.hourly_rate),
                currency: Set(data.currency.clone()),
                expert_rating: Set(Decimal::new(500, 2)),
                reviews_count: Set(0),
                timezone: Set(data.timezone.clone()),
                working_days: Set(data.working_days.clone()),
                work_start_time: Set(work_start_time),
                work_end_time: Set(work_end_time),
                allowed_session_durations: Set(data.allowed_session_durations.clone()),
                minimum_notice_minutes: Set(60),
                buffer_before_minutes: Set(0),
                buffer_after_minutes: Set(0),
                max_days_ahead: Set(30),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };

            active
                .insert(db)
                .await
                .map_err(|e| format!("failed to insert expert: {e}"))?
        }
    };

    Ok(UpsertExpertResponse {
        id: model.id,
        telegram_id: model.telegram_id,
        telegram_username: model.telegram_username,
        telegram_name: model.telegram_name,
        ton_wallet_address: model.ton_wallet_address,
        calendar_id: model.calendar_id,
        timezone: model.timezone,
        created,
    })
}

fn validate_data(data: &UpsertExpertData) -> Result<(), String> {
    if data.telegram_id <= 0 {
        return Err("telegram_id is required".to_string());
    }

    if data.username.trim().is_empty() {
        return Err("username is required".to_string());
    }

    if data.ton_wallet_address.trim().is_empty() {
        return Err("ton_wallet_address is required".to_string());
    }

    if data.timezone.trim().is_empty() {
        return Err("timezone is required".to_string());
    }

    if !data.working_days.is_array() {
        return Err("working_days must be a JSON array".to_string());
    }

    if !data.allowed_session_durations.is_array() {
        return Err("allowed_session_durations must be a JSON array".to_string());
    }

    Ok(())
}

fn build_telegram_name(first_name: &str, last_name: &str, username: &str) -> String {
    let full = [first_name.trim(), last_name.trim()]
        .into_iter()
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if !full.is_empty() {
        return full;
    }

    if !username.trim().is_empty() {
        return username.trim().to_string();
    }

    "Telegram user".to_string()
}

fn parse_time(value: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(value, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M"))
        .map_err(|_| format!("invalid time format: {value}"))
}