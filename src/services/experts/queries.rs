use sea_orm::{
    ColumnTrait,
    ConnectionTrait,
    EntityTrait,
    QueryFilter,
    QueryOrder,
    QuerySelect,
};

use crate::entities::experts;

use super::calendars::load_edit_calendar_options;
use super::dto::{EditExpertResponse, PopularExpertCardResponse, PublicExpertResponse};
use super::helpers::format_time;

pub async fn get_popular_experts<C>(
    db: &C,
    limit: u64,
) -> Result<Vec<PopularExpertCardResponse>, String>
where
    C: ConnectionTrait,
{
    let safe_limit = if limit == 0 { 6 } else { limit.min(12) };

    let experts = experts::Entity::find()
        .filter(experts::Column::IsActive.eq(true))
        .filter(experts::Column::IsBookable.eq(true))
        .order_by_desc(experts::Column::ReviewsCount)
        .order_by_desc(experts::Column::ExpertRating)
        .order_by_desc(experts::Column::UpdatedAt)
        .limit(safe_limit)
        .all(db)
        .await
        .map_err(|e| format!("failed to query popular experts: {e}"))?;

    Ok(experts
        .into_iter()
        .map(|expert| PopularExpertCardResponse {
            public_slug: expert.public_slug,
            display_name: expert.display_name,
            username: expert.username.unwrap_or_default(),
            photo_url: expert.photo_url,
            expert_rating: expert.expert_rating,
            reviews_count: expert.reviews_count,
        })
        .collect())
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
        telegram_id: expert.telegram_id,
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
        minimum_notice_minutes: expert.minimum_notice_minutes,
        buffer_before_minutes: expert.buffer_before_minutes,
        buffer_after_minutes: expert.buffer_after_minutes,
        max_days_ahead: expert.max_days_ahead,
        is_active: expert.is_active,
        is_bookable: expert.is_bookable,
        expert_rating: expert.expert_rating,
        reviews_count: expert.reviews_count,
    })
}