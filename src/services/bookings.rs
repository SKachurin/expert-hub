use chrono::{DateTime, Duration, FixedOffset, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    entities::{bookings, experts, payments},
    services::{
        availability::get_public_availability,
        experts::get_public_expert_by_slug,
    },
    state::AppState,
};

pub const BOOKING_STATUS_REQUESTED: &str = "requested";
pub const BOOKING_STATUS_AWAITING_PAYMENT: &str = "awaiting_payment";
pub const BOOKING_STATUS_FUNDED: &str = "funded";
pub const BOOKING_STATUS_WAITING_FOR_SESSION: &str = "waiting_for_session";
pub const BOOKING_STATUS_IN_GRACE_PERIOD: &str = "in_grace_period";
pub const BOOKING_STATUS_COMPLETED: &str = "completed";
pub const BOOKING_STATUS_EXPERT_NO_SHOW: &str = "expert_no_show";
pub const BOOKING_STATUS_CUSTOMER_NO_SHOW: &str = "customer_no_show";
pub const BOOKING_STATUS_REFUNDED: &str = "refunded";
pub const BOOKING_STATUS_REVIEW_OPEN: &str = "review_open";
pub const BOOKING_STATUS_CLOSED: &str = "closed";

pub const PAYMENT_STATUS_AWAITING_PAYMENT: &str = "awaiting_payment";
pub const PAYMENT_STATUS_FUNDED: &str = "funded";
pub const PAYMENT_STATUS_REFUNDED: &str = "refunded";
pub const PAYMENT_STATUS_SETTLED: &str = "settled";

pub const ACTIVE_BOOKING_BLOCKER_STATUSES: &[&str] = &[
    BOOKING_STATUS_REQUESTED,
    BOOKING_STATUS_AWAITING_PAYMENT,
    BOOKING_STATUS_FUNDED,
    BOOKING_STATUS_WAITING_FOR_SESSION,
    BOOKING_STATUS_IN_GRACE_PERIOD,
];

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
    pub expert_slug: String,
    pub slot_start: String,
    pub duration_minutes: i64,
    pub requested_by_telegram_id: i64,
    #[serde(default)]
    pub requested_by_username: Option<String>,
    #[serde(default)]
    pub requested_by_display_name: Option<String>,
    #[serde(default)]
    pub requested_by_ton_wallet: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BeginPaymentRequest {
    pub telegram_id: i64,
    pub ton_wallet_customer: String,
}

#[derive(Debug, Serialize)]
pub struct BookingSummaryResponse {
    pub id: i64,
    pub expert_id: i64,
    pub expert_slug: String,
    pub status: String,
    pub payment_status: Option<String>,
    pub requested_duration_minutes: i32,
    pub amount_quoted: Decimal,
    pub currency: String,
    pub slot_start: String,
    pub slot_end: String,
    pub payment_required_by: Option<String>,
    pub expires_at: String,
}

fn derive_amount(hourly_rate: Decimal, duration_minutes: i64) -> Result<Decimal, String> {
    if duration_minutes <= 0 {
        return Err("duration_minutes must be positive".to_string());
    }

    let duration = Decimal::from(duration_minutes);
    let sixty = Decimal::from(60);

    Ok((hourly_rate * duration) / sixty)
}

async fn ensure_requested_slot_is_still_available(
    state: &AppState,
    slug: &str,
    slot_start: DateTime<FixedOffset>,
    duration_minutes: i64,
) -> Result<(), String> {
    let expert = get_public_expert_by_slug(&state.db, slug.to_string()).await?;

    let availability = get_public_availability(
        &state.db,
        &state.config,
        &expert,
        0,
        Some(duration_minutes),
    )
    .await
    .map_err(|e| format!("failed to calculate availability: {e}"))?;

    let slot_start_rfc3339 = slot_start.to_rfc3339();

    let exists = availability
        .days
        .iter()
        .flat_map(|day| day.slots.iter())
        .any(|slot| slot.start_utc == slot_start_rfc3339);

    if !exists {
        return Err("selected slot is no longer available".to_string());
    }

    Ok(())
}

pub async fn create_booking_request(
    state: &AppState,
    body: CreateBookingRequest,
) -> Result<BookingSummaryResponse, String> {
    let slug = body.expert_slug.trim().to_lowercase();
    if slug.is_empty() {
        return Err("expert_slug is required".to_string());
    }

    if body.requested_by_telegram_id <= 0 {
        return Err("requested_by_telegram_id is required".to_string());
    }

    let slot_start = DateTime::parse_from_rfc3339(body.slot_start.trim())
        .map_err(|_| "slot_start must be a valid RFC3339 datetime".to_string())?;

    let expert = experts::Entity::find()
        .filter(experts::Column::PublicSlug.eq(slug.clone()))
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load expert: {e}"))?
        .ok_or_else(|| "expert not found".to_string())?;

    if !expert.is_active || !expert.is_bookable {
        return Err("expert is not bookable".to_string());
    }

    let expert_durations = expert.allowed_session_durations.clone();

    let duration_allowed = expert_durations
        .as_array()
        .map(|values| {
            values.iter().any(|v| {
                v.as_i64() == Some(body.duration_minutes) || v.as_u64() == Some(body.duration_minutes as u64)
            })
        })
        .unwrap_or(false);

    if !duration_allowed {
        return Err("selected duration is not allowed".to_string());
    }

    ensure_requested_slot_is_still_available(state, &slug, slot_start, body.duration_minutes).await?;

    let slot_end = slot_start + Duration::minutes(body.duration_minutes);
    let amount_quoted = derive_amount(expert.hourly_rate, body.duration_minutes)?;

    let now = Utc::now().fixed_offset();
    let expires_at = now + Duration::minutes(15);

    let booking = bookings::ActiveModel {
        expert_id: Set(expert.id),
        calendar_connection_id: Set(None),
        provider_used: Set(None),
        requested_by_telegram_id: Set(body.requested_by_telegram_id),
        requested_by_username: Set(body.requested_by_username.clone()),
        requested_by_display_name: Set(body.requested_by_display_name.clone()),
        requested_by_ton_wallet: Set(body.requested_by_ton_wallet.clone()),
        expert_timezone: Set(expert.timezone.clone()),
        requested_duration_minutes: Set(body.duration_minutes as i32),
        hourly_rate_snapshot: Set(expert.hourly_rate),
        amount_quoted: Set(amount_quoted),
        currency: Set(expert.currency.clone()),
        slot_start: Set(slot_start),
        slot_end: Set(slot_end),
        status: Set(BOOKING_STATUS_REQUESTED.to_string()),
        expert_confirmed_at: Set(None),
        expert_rejected_at: Set(None),
        rejected_reason: Set(None),
        payment_required_by: Set(Some(expires_at)),
        hold_expires_at: Set(Some(expires_at)),
        expires_at: Set(expires_at),
        external_booking_ref: Set(None),
        external_event_id: Set(None),
        external_event_url: Set(None),
        external_meeting_url: Set(None),
        external_join_url: Set(None),
        external_cancel_url: Set(None),
        external_sync_status: Set("not_synced".to_string()),
        external_synced_at: Set(None),
        external_sync_error: Set(None),
        session_started_at: Set(None),
        session_connected_at: Set(None),
        session_ended_at: Set(None),
        outcome_source: Set(None),
        metadata: Set(Some(json!({
            "public_slug": slug,
            "flow": "booking_request",
            "all_statuses_supported": [
                BOOKING_STATUS_REQUESTED,
                BOOKING_STATUS_AWAITING_PAYMENT,
                BOOKING_STATUS_FUNDED,
                BOOKING_STATUS_WAITING_FOR_SESSION,
                BOOKING_STATUS_IN_GRACE_PERIOD,
                BOOKING_STATUS_COMPLETED,
                BOOKING_STATUS_EXPERT_NO_SHOW,
                BOOKING_STATUS_CUSTOMER_NO_SHOW,
                BOOKING_STATUS_REFUNDED,
                BOOKING_STATUS_REVIEW_OPEN,
                BOOKING_STATUS_CLOSED
            ]
        }))),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(|e| format!("failed to create booking: {e}"))?;

    Ok(BookingSummaryResponse {
        id: booking.id,
        expert_id: booking.expert_id,
        expert_slug: slug,
        status: booking.status,
        payment_status: None,
        requested_duration_minutes: booking.requested_duration_minutes,
        amount_quoted: booking.amount_quoted,
        currency: booking.currency,
        slot_start: booking.slot_start.to_rfc3339(),
        slot_end: booking.slot_end.to_rfc3339(),
        payment_required_by: booking.payment_required_by.map(|v| v.to_rfc3339()),
        expires_at: booking.expires_at.to_rfc3339(),
    })
}

pub async fn begin_booking_payment(
    state: &AppState,
    booking_id: i64,
    body: BeginPaymentRequest,
) -> Result<BookingSummaryResponse, String> {
    let booking = bookings::Entity::find_by_id(booking_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load booking: {e}"))?
        .ok_or_else(|| "booking not found".to_string())?;

    if booking.requested_by_telegram_id != body.telegram_id {
        return Err("telegram user does not own this booking".to_string());
    }

    if booking.status != BOOKING_STATUS_REQUESTED && booking.status != BOOKING_STATUS_AWAITING_PAYMENT {
        return Err(format!("booking cannot move to payment from status {}", booking.status));
    }

    let expert = experts::Entity::find_by_id(booking.expert_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load expert: {e}"))?
        .ok_or_else(|| "expert not found".to_string())?;

    let now = Utc::now().fixed_offset();

    let mut booking_model: bookings::ActiveModel = booking.clone().into();
    booking_model.status = Set(BOOKING_STATUS_AWAITING_PAYMENT.to_string());
    booking_model.requested_by_ton_wallet = Set(Some(body.ton_wallet_customer.clone()));
    booking_model.updated_at = Set(now);

    let saved_booking = booking_model
        .update(&state.db)
        .await
        .map_err(|e| format!("failed to update booking: {e}"))?;

    let existing_payment = payments::Entity::find()
        .filter(payments::Column::BookingId.eq(saved_booking.id))
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to query payment: {e}"))?;

    let payment_status = if let Some(payment) = existing_payment {
        let mut payment_model: payments::ActiveModel = payment.into();
        payment_model.status = Set(PAYMENT_STATUS_AWAITING_PAYMENT.to_string());
        payment_model.ton_wallet_customer = Set(Some(body.ton_wallet_customer.clone()));
        payment_model.ton_wallet_expert = Set(Some(expert.ton_wallet_address.clone()));
        payment_model.updated_at = Set(now);
        payment_model
            .update(&state.db)
            .await
            .map_err(|e| format!("failed to update payment: {e}"))?
            .status
    } else {
        payments::ActiveModel {
            booking_id: Set(saved_booking.id),
            expert_id: Set(saved_booking.expert_id),
            customer_telegram_id: Set(saved_booking.requested_by_telegram_id),
            amount: Set(saved_booking.amount_quoted),
            currency: Set(saved_booking.currency.clone()),
            status: Set(PAYMENT_STATUS_AWAITING_PAYMENT.to_string()),
            ton_wallet_customer: Set(Some(body.ton_wallet_customer.clone())),
            ton_wallet_expert: Set(Some(expert.ton_wallet_address)),
            contract_address: Set(None),
            transaction_ref: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&state.db)
        .await
        .map_err(|e| format!("failed to create payment: {e}"))?
        .status
    };

    Ok(BookingSummaryResponse {
        id: saved_booking.id,
        expert_id: saved_booking.expert_id,
        expert_slug: saved_booking
            .metadata
            .as_ref()
            .and_then(|v| v.get("public_slug"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        status: saved_booking.status,
        payment_status: Some(payment_status),
        requested_duration_minutes: saved_booking.requested_duration_minutes,
        amount_quoted: saved_booking.amount_quoted,
        currency: saved_booking.currency,
        slot_start: saved_booking.slot_start.to_rfc3339(),
        slot_end: saved_booking.slot_end.to_rfc3339(),
        payment_required_by: saved_booking.payment_required_by.map(|v| v.to_rfc3339()),
        expires_at: saved_booking.expires_at.to_rfc3339(),
    })
}