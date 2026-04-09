use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set,
};

use crate::entities::calendar_connections;
use crate::entities::experts;

use super::dto::{
    UpdateExpertProfileRequest, UpsertExpertData, UpsertExpertRequest, UpsertExpertResponse,
};
use super::helpers::{
    build_display_name, generate_unique_public_slug, optional_trimmed_string, parse_time,
};
use super::queries::get_edit_expert_by_slug;
use super::validation::{validate_data, validate_update_data};

impl From<UpsertExpertRequest> for UpsertExpertData {
    fn from(value: UpsertExpertRequest) -> Self {
        Self {
            telegram_id: value.telegram_id,
            first_name: value.first_name,
            last_name: value.last_name,
            username: value.username,
            photo_url: value.photo_url,
            display_name: value.display_name,
            telegram_bio: value.telegram_bio,
            ton_wallet_address: value.ton_wallet_address,
            timezone: value.timezone,
            hourly_rate: value.hourly_rate,
            currency: value.currency,
            working_days: value.working_days,
            work_start_time: value.work_start_time,
            work_end_time: value.work_end_time,
            allowed_session_durations: value.allowed_session_durations,
            is_bookable: false,
        }
    }
}

pub async fn upsert_expert<C>(
    db: &C,
    data: UpsertExpertRequest,
) -> Result<UpsertExpertResponse, String>
where
    C: ConnectionTrait,
{
    upsert_expert_from_data(db, data.into()).await
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
            active.is_bookable = Set(data.is_bookable);
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
                currency: Set(data.currency.trim().to_uppercase()),
                expert_rating: Set(Decimal::new(500, 2)),
                reviews_count: Set(0),
                timezone: Set(data.timezone.clone()),
                working_days: Set(data.working_days.clone()),
                work_start_time: Set(work_start_time),
                work_end_time: Set(work_end_time),
                allowed_session_durations: Set(data.allowed_session_durations.clone()),
                is_bookable: Set(data.is_bookable),
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

pub async fn update_expert_profile_by_slug<C>(
    db: &C,
    slug: String,
    data: UpdateExpertProfileRequest,
) -> Result<super::dto::EditExpertResponse, String>
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

    if let Some(wallet) = data
        .ton_wallet_address
        .as_ref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    {
        active.ton_wallet_address = Set(wallet.to_string());
    }

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