use chrono::{NaiveTime, Utc};
use rand::{distributions::Alphanumeric, Rng};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entities::experts;
use crate::entities::calendar_connections;

#[derive(Debug, Deserialize)]
pub struct UpsertExpertRequest {
    pub telegram_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub photo_url: Option<String>,
    pub display_name: String,
    pub telegram_bio: Option<String>,
    pub ton_wallet_address: String,
    pub timezone: String,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub working_days: Value,
    pub work_start_time: String,
    pub work_end_time: String,
    pub allowed_session_durations: Value,
}

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
    pub username: String,
    pub display_name: String,
    pub ton_wallet_address: String,
    pub timezone: String,
    pub public_slug: String,
    pub created: bool,
}

#[derive(Debug, Serialize)]
pub struct EditCalendarOption {
    pub id: i64,
    pub provider: String,
    pub connection_label: String,
    pub is_primary: bool,
    pub is_enabled: bool,
    pub selected_calendar_name: Option<String>,
    pub selected_calendar_timezone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EditExpertResponse {
    pub id: i64,
    pub telegram_id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub display_name: String,
    pub telegram_bio: Option<String>,
    pub photo_url: Option<String>,
    pub public_slug: String,
    pub ton_wallet_address: String,
    pub timezone: String,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub working_days: Value,
    pub work_start_time: String,
    pub work_end_time: String,
    pub allowed_session_durations: Value,
    pub minimum_notice_minutes: i32,
    pub buffer_before_minutes: i32,
    pub buffer_after_minutes: i32,
    pub max_days_ahead: i32,
    pub is_active: bool,
    pub is_bookable: bool,
    pub booking_target_strategy: String,
    pub primary_calendar_connection_id: Option<i64>,
    pub calendar_connections: Vec<EditCalendarOption>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExpertProfileRequest {
    pub telegram_id: i64,
    pub display_name: String,
    pub telegram_bio: Option<String>,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub working_days: Value,
    pub work_start_time: String,
    pub work_end_time: String,
    pub allowed_session_durations: Value,
    pub minimum_notice_minutes: i32,
    pub buffer_before_minutes: i32,
    pub buffer_after_minutes: i32,
    pub max_days_ahead: i32,
    pub is_active: bool,
    pub is_bookable: bool,
    pub primary_calendar_connection_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PublicExpertResponse {
    pub id: i64,
    pub public_slug: String,
    pub display_name: String,
    pub username: String,
    pub photo_url: Option<String>,
    pub telegram_bio: Option<String>,
    pub timezone: String,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub allowed_session_durations: Value,
    pub working_days: Value,
    pub work_start_time: String,
    pub work_end_time: String,
    pub expert_rating: Decimal,
    pub reviews_count: i32,
    pub is_bookable: bool,
}

pub async fn upsert_expert<C>(
    db: &C,
    data: UpsertExpertRequest,
) -> Result<UpsertExpertResponse, String>
where
    C: ConnectionTrait,
{
    upsert_expert_from_data(
        db,
        UpsertExpertData {
            telegram_id: data.telegram_id,
            first_name: data.first_name,
            last_name: data.last_name,
            username: data.username,
            photo_url: data.photo_url,
            display_name: data.display_name,
            telegram_bio: data.telegram_bio,
            ton_wallet_address: data.ton_wallet_address,
            timezone: data.timezone,
            hourly_rate: data.hourly_rate,
            currency: data.currency,
            working_days: data.working_days,
            work_start_time: data.work_start_time,
            work_end_time: data.work_end_time,
            allowed_session_durations: data.allowed_session_durations,
        },
    )
    .await
}

pub async fn upsert_expert_from_data<C>(
    db: &C,
    data: UpsertExpertData,
) -> Result<UpsertExpertResponse, String>
where
    C: ConnectionTrait,
{
    validate_data(&data)?;

    let now = Utc::now().fixed_offset();
    let display_name = if data.display_name.trim().is_empty() {
        build_display_name(&data.first_name, &data.last_name, &data.username)
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

            active.first_name = Set(data.first_name.clone());
            active.last_name = Set(optional_trimmed_string(&data.last_name));
            active.username = Set(optional_trimmed_string(&data.username));
            active.display_name = Set(display_name.clone());
            active.telegram_bio = Set(data.telegram_bio.clone());
            active.ton_wallet_address = Set(data.ton_wallet_address.clone());
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
            let public_slug =
                generate_unique_public_slug(db, &data.username, &display_name).await?;

            let active = experts::ActiveModel {
                telegram_id: Set(data.telegram_id),
                first_name: Set(data.first_name.clone()),
                last_name: Set(optional_trimmed_string(&data.last_name)),
                username: Set(optional_trimmed_string(&data.username)),
                display_name: Set(display_name.clone()),
                telegram_bio: Set(data.telegram_bio.clone()),
                ton_wallet_address: Set(data.ton_wallet_address.clone()),
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
                public_slug: Set(public_slug),
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
        username: model.username.unwrap_or_default(),
        display_name: model.display_name,
        ton_wallet_address: model.ton_wallet_address,
        timezone: model.timezone,
        public_slug: model.public_slug,
        created,
    })
}

pub async fn get_edit_expert_by_slug<C>(
    db: &C,
    slug: String,
) -> Result<EditExpertResponse, String>
where
    C: ConnectionTrait,
{
    let slug = slug.trim().to_string();
    if slug.is_empty() {
        return Err("slug is required".to_string());
    }

    let expert = experts::Entity::find()
        .filter(experts::Column::PublicSlug.eq(slug))
        .one(db)
        .await
        .map_err(|e| format!("failed to query expert: {e}"))?
        .ok_or_else(|| "expert not found".to_string())?;

    let calendar_connections = load_edit_calendar_options(db, expert.id).await?;
    let primary_calendar_connection_id = calendar_connections
        .iter()
        .find(|item| item.is_primary)
        .map(|item| item.id);

    Ok(EditExpertResponse {
        id: expert.id,
        telegram_id: expert.telegram_id,
        first_name: expert.first_name.clone(),
        last_name: expert.last_name.clone(),
        username: expert.username.clone(),
        display_name: expert.display_name.clone(),
        telegram_bio: expert.telegram_bio.clone(),
        photo_url: expert.photo_url.clone(),
        public_slug: expert.public_slug.clone(),
        ton_wallet_address: expert.ton_wallet_address.clone(),
        timezone: expert.timezone.clone(),
        hourly_rate: expert.hourly_rate,
        currency: expert.currency.clone(),
        working_days: expert.working_days.clone(),
        work_start_time: format_time(&expert.work_start_time),
        work_end_time: format_time(&expert.work_end_time),
        allowed_session_durations: expert.allowed_session_durations.clone(),
        minimum_notice_minutes: expert.minimum_notice_minutes,
        buffer_before_minutes: expert.buffer_before_minutes,
        buffer_after_minutes: expert.buffer_after_minutes,
        max_days_ahead: expert.max_days_ahead,
        is_active: expert.is_active,
        is_bookable: expert.is_bookable,
        booking_target_strategy: expert.booking_target_strategy.clone(),
        primary_calendar_connection_id,
        calendar_connections,
    })
}

pub async fn update_expert_profile_by_slug<C>(
    db: &C,
    slug: String,
    data: UpdateExpertProfileRequest,
) -> Result<EditExpertResponse, String>
where
    C: ConnectionTrait,
{
    validate_update_data(&data)?;

    let slug = slug.trim().to_string();
    if slug.is_empty() {
        return Err("slug is required".to_string());
    }

    let existing = experts::Entity::find()
        .filter(experts::Column::PublicSlug.eq(slug.clone()))
        .one(db)
        .await
        .map_err(|e| format!("failed to query expert: {e}"))?
        .ok_or_else(|| "expert not found".to_string())?;

    if existing.telegram_id != data.telegram_id {
        return Err("telegram user does not match this expert".to_string());
    }

    let work_start_time = parse_time(&data.work_start_time)?;
    let work_end_time = parse_time(&data.work_end_time)?;
    let now = Utc::now().fixed_offset();

    let mut active: experts::ActiveModel = existing.into();
    active.display_name = Set(data.display_name.trim().to_string());
    active.telegram_bio = Set(
        data.telegram_bio
            .as_ref()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty()),
    );
    active.hourly_rate = Set(data.hourly_rate);
    active.currency = Set(data.currency.trim().to_uppercase());
    active.working_days = Set(data.working_days.clone());
    active.work_start_time = Set(work_start_time);
    active.work_end_time = Set(work_end_time);
    active.allowed_session_durations = Set(data.allowed_session_durations.clone());
    active.minimum_notice_minutes = Set(data.minimum_notice_minutes);
    active.buffer_before_minutes = Set(data.buffer_before_minutes);
    active.buffer_after_minutes = Set(data.buffer_after_minutes);
    active.max_days_ahead = Set(data.max_days_ahead);
    active.is_active = Set(data.is_active);
    active.is_bookable = Set(data.is_bookable);

    // hidden from UI, always fixed
    active.calendar_conflict_mode = Set("all_enabled_busy".to_string());
    active.booking_target_strategy = Set("primary_calendar".to_string());
    active.updated_at = Set(now);

    let saved = active
        .update(db)
        .await
        .map_err(|e| format!("failed to update expert: {e}"))?;

    if let Some(primary_id) = data.primary_calendar_connection_id {
        let rows = calendar_connections::Entity::find()
            .filter(calendar_connections::Column::ExpertId.eq(saved.id))
            .all(db)
            .await
            .map_err(|e| format!("failed to load calendar connections: {e}"))?;

        for row in rows {
            let mut active_row: calendar_connections::ActiveModel = row.into();
            active_row.is_primary = Set(active_row.id.clone().unwrap() == primary_id);
            active_row.updated_at = Set(now);

            active_row
                .update(db)
                .await
                .map_err(|e| format!("failed to update calendar primary flag: {e}"))?;
        }
    }

    get_edit_expert_by_slug(db, slug).await
}

pub async fn get_public_expert_by_slug<C>(
    db: &C,
    slug: String,
) -> Result<PublicExpertResponse, String>
where
    C: ConnectionTrait,
{
    let slug = slug.trim().to_lowercase();

    if slug.is_empty() {
        return Err("slug is required".to_string());
    }

    let expert = experts::Entity::find()
        .filter(experts::Column::PublicSlug.eq(slug))
        .one(db)
        .await
        .map_err(|e| format!("failed to query expert: {e}"))?
        .ok_or_else(|| "expert not found".to_string())?;

    Ok(PublicExpertResponse {
        id: expert.id,
        public_slug: expert.public_slug,
        display_name: expert.display_name,
        username: expert.username.unwrap_or_default(),
        photo_url: expert.photo_url,
        telegram_bio: expert.telegram_bio,
        timezone: expert.timezone,
        hourly_rate: expert.hourly_rate,
        currency: expert.currency,
        allowed_session_durations: expert.allowed_session_durations,
        working_days: expert.working_days,
        work_start_time: expert.work_start_time.format("%H:%M").to_string(),
        work_end_time: expert.work_end_time.format("%H:%M").to_string(),
        expert_rating: expert.expert_rating,
        reviews_count: expert.reviews_count,
        is_bookable: expert.is_bookable,
    })
}
fn format_time(value: &NaiveTime) -> String {
    value.format("%H:%M").to_string()
}

fn build_calendar_label(model: &calendar_connections::Model) -> String {
    if let Some(label) = &model.connection_label {
        if !label.trim().is_empty() {
            return label.clone();
        }
    }

    match (
        model.provider.as_str(),
        model.account_email.as_deref(),
        model.selected_calendar_name.as_deref(),
    ) {
        ("google", Some(email), Some(name)) => format!("Google · {} · {}", email, name),
        ("google", Some(email), None) => format!("Google · {}", email),
        ("google", None, Some(name)) => format!("Google · {}", name),
        _ => model.provider.clone(),
    }
}

async fn load_edit_calendar_options<C: ConnectionTrait>(
    db: &C,
    expert_id: i64,
) -> Result<Vec<EditCalendarOption>, String> {
    let rows = calendar_connections::Entity::find()
        .filter(calendar_connections::Column::ExpertId.eq(expert_id))
        .order_by_desc(calendar_connections::Column::IsPrimary)
        .order_by_asc(calendar_connections::Column::Id)
        .all(db)
        .await
        .map_err(|e| format!("failed to load calendar connections: {e}"))?;

    Ok(rows
        .into_iter()
        .map(|row| EditCalendarOption {
            id: row.id,
            provider: row.provider.clone(),
            connection_label: build_calendar_label(&row),
            is_primary: row.is_primary,
            is_enabled: row.is_enabled,
            selected_calendar_name: row.selected_calendar_name.clone(),
            selected_calendar_timezone: row.selected_calendar_timezone.clone(),
        })
        .collect())
}

fn validate_update_data(data: &UpdateExpertProfileRequest) -> Result<(), String> {
    if data.telegram_id <= 0 {
        return Err("telegram_id is required".to_string());
    }

    if data.display_name.trim().is_empty() {
        return Err("display_name is required".to_string());
    }

    if data.currency.trim().is_empty() {
        return Err("currency is required".to_string());
    }

    if !data.working_days.is_array() {
        return Err("working_days must be a JSON array".to_string());
    }

    if !data.allowed_session_durations.is_array() {
        return Err("allowed_session_durations must be a JSON array".to_string());
    }

    if data.minimum_notice_minutes < 0
        || data.buffer_before_minutes < 0
        || data.buffer_after_minutes < 0
        || data.max_days_ahead < 1
    {
        return Err("invalid scheduling numbers".to_string());
    }

    Ok(())
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

fn build_display_name(first_name: &str, last_name: &str, username: &str) -> String {
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

fn optional_trimmed_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn slugify_part(input: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in input.trim().to_lowercase().chars() {
        let mapped = match ch {
            'a'..='z' | '0'..='9' => Some(ch),
            '_' | ' ' | '-' => Some('-'),
            _ => None,
        };

        if let Some(c) = mapped {
            if c == '-' {
                if !prev_dash && !out.is_empty() {
                    out.push('-');
                    prev_dash = true;
                }
            } else {
                out.push(c);
                prev_dash = false;
            }
        }
    }

    out.trim_matches('-').to_string()
}

fn short_suffix() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}

async fn generate_unique_public_slug<C: ConnectionTrait>(
    db: &C,
    username: &str,
    display_name: &str,
) -> Result<String, String> {
    let mut base = if !username.trim().is_empty() {
        slugify_part(username)
    } else {
        slugify_part(display_name)
    };

    if base.is_empty() {
        base = "expert".to_string();
    }

    let exists = experts::Entity::find()
        .filter(experts::Column::PublicSlug.eq(base.clone()))
        .one(db)
        .await
        .map_err(|e| format!("failed to check public slug: {e}"))?;

    if exists.is_none() {
        return Ok(base);
    }

    for _ in 0..20 {
        let candidate = format!("{}-{}", base, short_suffix());

        let exists = experts::Entity::find()
            .filter(experts::Column::PublicSlug.eq(candidate.clone()))
            .one(db)
            .await
            .map_err(|e| format!("failed to check public slug: {e}"))?;

        if exists.is_none() {
            return Ok(candidate);
        }
    }

    Err("failed to generate unique public slug".to_string())
}