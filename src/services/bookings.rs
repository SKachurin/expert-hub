use actix_web::web;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, TransactionTrait, QueryFilter, Set,
};
use sea_orm::sea_query::Expr;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::{sleep, Duration as TokioDuration};
use crate::services::ton::payment_metadata::TonPaymentMetadata;

use crate::{
    entities::{bookings, experts, payments},
    services::{
        availability::get_public_availability,
        booking_rules::{
            ensure_slot_has_platform_min_lead,
            REJECTED_REASON_EXPERT_DECLINED,
            REJECTED_REASON_EXPERT_RESPONSE_TIMEOUT,
        },
        experts::get_public_expert_by_slug,
        telegram_bot::TelegramBotClient,
        ton::{
            client::TonWorkerClient,
            controller::TonController,
            rates::{convert_usd_to_ton_amount, fetch_ton_usd_rate},
        },
    },
    state::AppState,
};

pub const BOOKING_STATUS_REQUESTED: &str = "requested";
pub const BOOKING_STATUS_AWAITING_PAYMENT: &str = "awaiting_payment";
pub const BOOKING_STATUS_FUNDED: &str = "funded";
pub const BOOKING_STATUS_PROCESSING: &str = "processing";
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
pub const PAYMENT_STATUS_SUBMITTED: &str = "submitted";

pub const ACTIVE_BOOKING_BLOCKER_STATUSES: &[&str] = &[
    BOOKING_STATUS_REQUESTED,
    BOOKING_STATUS_AWAITING_PAYMENT,
    BOOKING_STATUS_FUNDED,
    BOOKING_STATUS_PROCESSING,
    BOOKING_STATUS_WAITING_FOR_SESSION,
    BOOKING_STATUS_IN_GRACE_PERIOD,
];

const STATE_WAITING_SESSION: i32 = 2;
const STATE_REFUNDED_TO_CUSTOMER: i32 = 4;

const BALANCE_TOLERANCE_BPS: u128 = 30; // 30 basis points = 0.3%

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

    pub payment_id: Option<i64>,
    pub contract_address: Option<String>,
    pub state_init_boc: Option<String>,
    pub amount_nano_ton: Option<String>,

    pub requested_duration_minutes: i32,
    pub amount_quoted: Decimal,
    pub currency: String,
    pub slot_start: String,
    pub slot_end: String,
    pub payment_required_by: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct BeginPaymentResponse {
    pub booking_id: i64,
    pub payment_id: i64,
    pub contract_address: String,
    pub destination_address: String,
    pub state_init_boc: String,
    pub amount_nano_ton: String,
    pub customer_total_ton: String,
    pub customer_total_nano_ton: String,
    pub recommended_gas_buffer_nano_ton: String,
    pub total_deploy_value_nano_ton: String,
    pub payment_status: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmBookingPaymentRequest {
    pub boc: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConfirmBookingPaymentResponse {
    pub booking_id: i64,
    pub payment_id: i64,
    pub booking_status: String,
    pub payment_status: String,
    pub contract_address: String,
    pub transaction_ref: Option<String>,
    pub verified: bool,
    pub contract_state: Option<i32>,
    pub account_state: String,
    pub balance_nano_ton: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpertBookingDecision {
    Confirm,
    Decline,
}

pub async fn process_expert_booking_decision(
    state: &web::Data<AppState>,
    booking_id: i64,
    payment_id: i64,
    expert_telegram_id: i64,
    decision: ExpertBookingDecision,
) -> Result<BookingSummaryResponse, String> {

    println!(
        "BOOKING_DECISION_START booking={} payment={} expert={} decision={:?}",
        booking_id,
        payment_id,
        expert_telegram_id,
        decision
    );

    let booking = bookings::Entity::find_by_id(booking_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load booking: {e}"))?
        .ok_or_else(|| "booking not found".to_string())?;

    let expert = experts::Entity::find_by_id(booking.expert_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load expert: {e}"))?
        .ok_or_else(|| "expert not found".to_string())?;

    if expert.telegram_id != expert_telegram_id {
        return Err("only the expert can confirm or decline this booking".to_string());
    }

    if booking.status == BOOKING_STATUS_REFUNDED
        && booking.rejected_reason.as_deref() == Some(REJECTED_REASON_EXPERT_RESPONSE_TIMEOUT)
    {
        return Err("Too late. This request has already been refunded.".to_string());
    }

    if booking.status != BOOKING_STATUS_FUNDED {
        return Err(format!(
            "booking must be funded before expert decision, current status: {}",
            booking.status
        ));
    }

    if !claim_processing(&state.db, booking.id).await? {
        let current = bookings::Entity::find_by_id(booking.id)
            .one(&state.db)
            .await
            .map_err(|e| format!("failed to reload booking: {e}"))?
            .ok_or_else(|| "booking not found".to_string())?;

        match current.status.as_str() {
            BOOKING_STATUS_WAITING_FOR_SESSION => {
                return summarize_booking(&state.db, current.id).await;
            }

            BOOKING_STATUS_REFUNDED => {
                return Err("Too late. This request has already been refunded.".to_string());
            }

            BOOKING_STATUS_PROCESSING => {
                return Err("Booking is already being processed.".to_string());
            }

            _ => {
                return Err(format!(
                    "booking already moved to {}",
                    current.status
                ));
            }
        }
    }

    let booking = bookings::Entity::find_by_id(booking.id)
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to reload booking: {e}"))?
        .ok_or_else(|| "booking not found".to_string())?;

    let payment = payments::Entity::find()
        .filter(payments::Column::Id.eq(payment_id))
        .filter(payments::Column::BookingId.eq(booking.id))
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load payment: {e}"))?
        .ok_or_else(|| "payment not found".to_string())?;

    if payment.status != PAYMENT_STATUS_FUNDED {
        return Err(format!(
            "payment must be funded before expert decision, current status: {}",
            payment.status
        ));
    }

    let now = Utc::now().fixed_offset();

    if booking.expires_at <= now {
        expire_unanswered_funded_booking(state.get_ref(), &booking, &payment).await?;

        return Err("Too late. This request has already been refunded.".to_string());
    }

    let contract_address = payment
        .contract_address
        .clone()
        .ok_or_else(|| "payment has no contract address".to_string())?;

    let ton_client = TonWorkerClient::new(
        state.config.ton_worker_base_url.clone(),
        state.config.ton_worker_auth_token.clone(),
    );
    let ton_controller = TonController::new(ton_client);

     match decision {
            ExpertBookingDecision::Confirm => {
                // Target on-chain state after successful expert_confirm
                if contract_already_in_state(
                    &ton_controller,
                    &contract_address,
                    STATE_WAITING_SESSION,
                )
                .await?
                {
                    tracing::info!(
                        booking_id = booking.id,
                        "expert_confirm already applied on-chain; treating as success"
                    );
                } else {
                    match ton_controller
                        .confirm_expert(contract_address.clone(), payment.id, booking.id)
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            // Message may have landed even if HTTP timed out
                            if contract_already_in_state(
                                &ton_controller,
                                &contract_address,
                                STATE_WAITING_SESSION,
                            )
                            .await?
                            {
                                tracing::warn!(
                                    booking_id = booking.id,
                                    error = %e,
                                    "expert_confirm HTTP failed but on-chain state is already waiting_for_session; treating as success"
                                );
                            } else {
                                rollback_processing_to_funded(&state.db, booking.id).await?;
                                return Err(e);
                            }
                        }
                    }
                }

                let now = Utc::now().fixed_offset();

                let mut active: bookings::ActiveModel = booking.clone().into();
                active.status = Set(BOOKING_STATUS_WAITING_FOR_SESSION.to_string());
                active.expert_confirmed_at = Set(Some(now));
                active.updated_at = Set(now);

                let updated = active
                    .update(&state.db)
                    .await
                    .map_err(|e| format!("failed to update booking after expert confirm: {e}"))?;

                let bot = TelegramBotClient::new(state.config.active_telegram_bot_token.clone());
                if let Err(err) = bot.notify_customer_expert_confirmed(&expert, &updated).await {
                    eprintln!(
                        "[telegram] failed to notify customer about expert confirmation: {err}"
                    );
                }

                summarize_booking(&state.db, updated.id).await
            }

            ExpertBookingDecision::Decline => {
                // Target on-chain state after successful expert_decline / refund
                if contract_already_in_state(
                    &ton_controller,
                    &contract_address,
                    STATE_REFUNDED_TO_CUSTOMER,
                )
                .await?
                {
                    tracing::info!(
                        booking_id = booking.id,
                        "expert_decline already applied on-chain; treating as success"
                    );
                } else {
                    match ton_controller
                        .decline_expert(
                            contract_address.clone(),
                            payment.id,
                            booking.id,
                            Some("expert declined booking".to_string()),
                        )
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            if contract_already_in_state(
                                &ton_controller,
                                &contract_address,
                                STATE_REFUNDED_TO_CUSTOMER,
                            )
                            .await?
                            {
                                tracing::warn!(
                                    booking_id = booking.id,
                                    error = %e,
                                    "expert_decline HTTP failed but on-chain state is already refunded; treating as success"
                                );
                            } else {
                                rollback_processing_to_funded(&state.db, booking.id).await?;
                                return Err(e);
                            }
                        }
                    }
                }

                let now = Utc::now().fixed_offset();

                let mut active_booking: bookings::ActiveModel = booking.clone().into();
                active_booking.status = Set(BOOKING_STATUS_REFUNDED.to_string());
                active_booking.expert_rejected_at = Set(Some(now));
                active_booking.rejected_reason =
                    Set(Some(REJECTED_REASON_EXPERT_DECLINED.to_string()));
                active_booking.updated_at = Set(now);

                let txn = state
                    .db
                    .begin()
                    .await
                    .map_err(|e| format!("failed to begin transaction: {e}"))?;

                let updated_booking = active_booking
                    .update(&txn)
                    .await
                    .map_err(|e| format!("failed to mark booking as refunded: {e}"))?;

                let mut active_payment: payments::ActiveModel = payment.clone().into();
                active_payment.status = Set(PAYMENT_STATUS_REFUNDED.to_string());
                active_payment.updated_at = Set(now);

                active_payment
                    .update(&txn)
                    .await
                    .map_err(|e| format!("failed to mark payment as refunded: {e}"))?;

                txn.commit()
                    .await
                    .map_err(|e| format!("failed to commit transaction: {e}"))?;

                let bot = TelegramBotClient::new(state.config.active_telegram_bot_token.clone());
                if let Err(err) = bot.notify_customer_expert_declined(&updated_booking).await {
                    eprintln!("[telegram] failed to notify customer about expert decline: {err}");
                }

                summarize_booking(&state.db, updated_booking.id).await
            }
     }
}

pub async fn expire_unanswered_funded_booking(
    state: &AppState,
    booking: &bookings::Model,
    payment: &payments::Model,
) -> Result<(), String> {
    if booking.status != BOOKING_STATUS_FUNDED {
        return Ok(());
    }

    if payment.status != PAYMENT_STATUS_FUNDED {
        return Ok(());
    }

    // Atomically claim this booking.
    // If another process (expert confirm/decline) already claimed it,
    // there is nothing left for the timeout worker to do.
    if !claim_processing(&state.db, booking.id).await? {
        return Ok(());
    }

    let contract_address = payment
        .contract_address
        .clone()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "payment has no contract address".to_string())?;

    let ton_client = TonWorkerClient::new(
        state.config.ton_worker_base_url.clone(),
        state.config.ton_worker_auth_token.clone(),
    );

    let ton_controller = TonController::new(ton_client);

    if contract_already_in_state(
        &ton_controller,
        &contract_address,
        STATE_REFUNDED_TO_CUSTOMER,
    )
    .await?
    {
        tracing::info!(
            booking_id = booking.id,
            "timeout refund already applied on-chain"
        );
    } else {
        match ton_controller
            .decline_expert(
                contract_address.clone(),
                payment.id,
                booking.id,
                Some("expert response timeout".to_string()),
            )
            .await
        {
            Ok(_) => {}

            Err(e) => {
                if contract_already_in_state(
                    &ton_controller,
                    &contract_address,
                    STATE_REFUNDED_TO_CUSTOMER,
                )
                .await?
                {
                    tracing::warn!(
                        booking_id = booking.id,
                        error = %e,
                        "timeout refund HTTP failed but contract is already refunded"
                    );
                } else {
                    rollback_processing_to_funded(&state.db, booking.id).await?;
                    return Err(e);
                }
            }
        }
    }

    let now = Utc::now().fixed_offset();

    let mut active_booking: bookings::ActiveModel = booking.clone().into();
    active_booking.status = Set(BOOKING_STATUS_REFUNDED.to_string());
    active_booking.expert_rejected_at = Set(Some(now));
    active_booking.rejected_reason =
        Set(Some(REJECTED_REASON_EXPERT_RESPONSE_TIMEOUT.to_string()));
    active_booking.updated_at = Set(now);

    let txn = state
        .db
        .begin()
        .await
        .map_err(|e| format!("failed to begin transaction: {e}"))?;

    let updated_booking = active_booking
        .update(&txn)
        .await
        .map_err(|e| format!("failed to mark booking as timeout/refunded: {e}"))?;

    let mut active_payment: payments::ActiveModel = payment.clone().into();
    active_payment.status = Set(PAYMENT_STATUS_REFUNDED.to_string());
    active_payment.updated_at = Set(now);

    active_payment
        .update(&txn)
        .await
        .map_err(|e| format!("failed to mark payment as timeout/refunded: {e}"))?;

    txn.commit()
        .await
        .map_err(|e| format!("failed to commit transaction: {e}"))?;

    let Some(expert) = experts::Entity::find_by_id(updated_booking.expert_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load expert for timeout notification: {e}"))?
    else {
        return Ok(());
    };

    let bot = TelegramBotClient::new(state.config.active_telegram_bot_token.clone());

    if let Err(err) = bot
        .notify_customer_expert_response_timeout_refunded(&updated_booking)
        .await
    {
        eprintln!(
            "[telegram] failed to notify customer about expert response timeout refund: {err}"
        );
    }

    if let Err(err) = bot
        .notify_expert_response_timeout_refunded(&expert, &updated_booking)
        .await
    {
        eprintln!(
            "[telegram] failed to notify expert about expert response timeout refund: {err}"
        );
    }

    Ok(())
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
    ensure_slot_has_platform_min_lead(slot_start)?;

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
                v.as_i64() == Some(body.duration_minutes)
                    || v.as_u64() == Some(body.duration_minutes as u64)
            })
        })
        .unwrap_or(false);

    if !duration_allowed {
        return Err("selected duration is not allowed".to_string());
    }

    ensure_requested_slot_is_still_available(
        state,
        &slug,
        slot_start,
        body.duration_minutes,
    )
    .await?;

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

        payment_id: None,
        contract_address: None,
        state_init_boc: None,
        amount_nano_ton: None,

        requested_duration_minutes: booking.requested_duration_minutes,
        amount_quoted: booking.amount_quoted,
        currency: booking.currency,
        slot_start: booking.slot_start.to_rfc3339(),
        slot_end: booking.slot_end.to_rfc3339(),
        payment_required_by: booking.payment_required_by.map(|v| v.to_rfc3339()),
        expires_at: booking.expires_at.to_rfc3339(),
    })
}

fn ton_decimal_string_to_nano_ton(value: &str) -> Result<String, String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("TON amount is empty".to_string());
    }

    let mut parts = trimmed.split('.');

    let whole = parts
        .next()
        .ok_or_else(|| "TON amount whole part is missing".to_string())?;

    let fraction = parts.next().unwrap_or("");

    if parts.next().is_some() {
        return Err("TON amount has more than one decimal point".to_string());
    }

    if whole.is_empty() || !whole.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("invalid TON amount whole part: {trimmed}"));
    }

    if fraction.len() > 9 || !fraction.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("invalid TON amount fractional part: {trimmed}"));
    }

    let mut fraction_padded = fraction.to_string();
    while fraction_padded.len() < 9 {
        fraction_padded.push('0');
    }

    let combined = format!("{whole}{fraction_padded}");
    let normalized = combined.trim_start_matches('0');

    Ok(if normalized.is_empty() {
        "0".to_string()
    } else {
        normalized.to_string()
    })
}

fn parse_nano_ton(value: &str, field: &str) -> Result<u128, String> {
    value
        .trim()
        .parse::<u128>()
        .map_err(|e| format!("failed to parse {field} as nanoTON: {e}"))
}


async fn summarize_booking(
    db: &DatabaseConnection,
    booking_id: i64,
) -> Result<BookingSummaryResponse, String> {
    let booking = bookings::Entity::find_by_id(booking_id)
        .one(db)
        .await
        .map_err(|e| format!("failed to load booking summary: {e}"))?
        .ok_or_else(|| "booking not found".to_string())?;

    let expert = experts::Entity::find_by_id(booking.expert_id)
        .one(db)
        .await
        .map_err(|e| format!("failed to load expert for booking summary: {e}"))?
        .ok_or_else(|| "expert not found".to_string())?;

    let payment = payments::Entity::find()
        .filter(payments::Column::BookingId.eq(booking.id))
        .one(db)
        .await
        .map_err(|e| format!("failed to load payment for booking summary: {e}"))?;

    let (payment_status, payment_id, contract_address, amount_nano_ton) =
        if let Some(payment) = payment {
            let amount_nano_ton = booking
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("ton_payment"))
                .and_then(|ton_payment| ton_payment.get("wallet_send_amount_nano_ton"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());

            (
                Some(payment.status),
                Some(payment.id),
                payment.contract_address,
                amount_nano_ton,
            )
        } else {
            (None, None, None, None)
        };

    Ok(BookingSummaryResponse {
        id: booking.id,
        expert_id: booking.expert_id,
        expert_slug: expert.public_slug,
        status: booking.status,
        payment_status,

        payment_id,
        contract_address,
        state_init_boc: None,
        amount_nano_ton,

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
) -> Result<BeginPaymentResponse, String> {
    let booking = bookings::Entity::find_by_id(booking_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load booking: {e}"))?
        .ok_or_else(|| "booking not found".to_string())?;

    if booking.requested_by_telegram_id != body.telegram_id {
        return Err("telegram user does not own this booking".to_string());
    }

    if booking.status != BOOKING_STATUS_REQUESTED
        && booking.status != BOOKING_STATUS_AWAITING_PAYMENT
    {
        return Err(format!(
            "booking cannot move to payment from status {}",
            booking.status
        ));
    }

    ensure_slot_has_platform_min_lead(booking.slot_start)?;

    let expert = experts::Entity::find_by_id(booking.expert_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load expert: {e}"))?
        .ok_or_else(|| "expert not found".to_string())?;

    if expert.ton_wallet_address.trim().is_empty() {
        return Err("expert TON wallet is missing".to_string());
    }

    if body.ton_wallet_customer.trim().is_empty() {
        return Err("customer TON wallet is missing".to_string());
    }

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

    let saved_payment = if let Some(payment) = existing_payment {
        if let Some(contract_address) = payment.contract_address.clone() {
            if !contract_address.trim().is_empty() {
                let metadata = saved_booking
                    .metadata
                    .clone()
                    .unwrap_or_else(|| json!({}));

                let ton = metadata
                    .get("ton_payment")
                    .unwrap_or(&serde_json::Value::Null);

                return Ok(BeginPaymentResponse {
                    booking_id: saved_booking.id,
                    payment_id: payment.id,
                    contract_address: contract_address.clone(),
                    destination_address: contract_address,
                    state_init_boc: ton
                        .get("state_init_boc")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    amount_nano_ton: ton
                        .get("wallet_send_amount_nano_ton")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    customer_total_ton: "".to_string(),
                    customer_total_nano_ton: ton
                        .get("customer_total_nano_ton")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    recommended_gas_buffer_nano_ton: ton
                        .get("recommended_gas_buffer_nano_ton")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    total_deploy_value_nano_ton: ton
                        .get("total_deploy_value_nano_ton")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    payment_status: payment.status.clone(),
                });
            }
        }

        let mut payment_model: payments::ActiveModel = payment.into();

        payment_model.status = Set(PAYMENT_STATUS_AWAITING_PAYMENT.to_string());
        payment_model.ton_wallet_customer = Set(Some(body.ton_wallet_customer.clone()));
        payment_model.ton_wallet_expert = Set(Some(expert.ton_wallet_address.clone()));
        payment_model.updated_at = Set(now);

        payment_model
            .update(&state.db)
            .await
            .map_err(|e| format!("failed to update payment: {e}"))?
    } else {
        payments::ActiveModel {
            booking_id: Set(saved_booking.id),
            expert_id: Set(saved_booking.expert_id),
            customer_telegram_id: Set(saved_booking.requested_by_telegram_id),
            amount: Set(saved_booking.amount_quoted),
            currency: Set(saved_booking.currency.clone()),
            status: Set(PAYMENT_STATUS_AWAITING_PAYMENT.to_string()),
            ton_wallet_customer: Set(Some(body.ton_wallet_customer.clone())),
            ton_wallet_expert: Set(Some(expert.ton_wallet_address.clone())),
            contract_address: Set(None),
            transaction_ref: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&state.db)
        .await
        .map_err(|e| format!("failed to create payment: {e}"))?
    };

    let escrow_amount_ton = if saved_booking.currency == "USD" {
        let ton_usd_rate = fetch_ton_usd_rate().await?;
        convert_usd_to_ton_amount(saved_booking.amount_quoted, ton_usd_rate)?
    } else if saved_booking.currency == "TON" {
        saved_booking.amount_quoted
    } else {
        return Err(format!(
            "unsupported booking currency for TON payment: {}",
            saved_booking.currency
        ));
    };

    let expert_amount_ton = escrow_amount_ton.round_dp(9);

    if expert_amount_ton <= Decimal::ZERO {
        return Err("calculated expert amount in TON must be positive".to_string());
    }

    let platform_fee_ton = (expert_amount_ton * state.config.platform_fee_percent
        / Decimal::from(100))
        .round_dp(9);
    let gas_reserve_ton = state.config.ton_gas_reserve.round_dp(9);
    let controller_reserve_ton = state.config.ton_controller_action_reserve.round_dp(9);

    let customer_total_ton =
        (expert_amount_ton + platform_fee_ton + gas_reserve_ton + controller_reserve_ton)
            .round_dp(9);

    if customer_total_ton <= Decimal::ZERO {
        return Err("calculated customer total in TON must be positive".to_string());
    }


    let expert_amount_nano: u128 = parse_nano_ton(
        &ton_decimal_string_to_nano_ton(&expert_amount_ton.normalize().to_string())?,
        "expert_amount_nano_ton",
    )?;

    let platform_fee_nano: u128 = parse_nano_ton(
        &ton_decimal_string_to_nano_ton(&platform_fee_ton.normalize().to_string())?,
        "platform_fee_nano_ton",
    )?;

    let gas_reserve_nano: u128 = parse_nano_ton(
        &ton_decimal_string_to_nano_ton(&gas_reserve_ton.normalize().to_string())?,
        "gas_reserve_nano_ton",
    )?;

    let controller_reserve_nano: u128 = parse_nano_ton(
        &ton_decimal_string_to_nano_ton(&controller_reserve_ton.normalize().to_string())?,
        "controller_reserve_nano_ton",
    )?;

    let customer_total_nano = expert_amount_nano
        .checked_add(platform_fee_nano)
        .and_then(|v| v.checked_add(gas_reserve_nano))
        .and_then(|v| v.checked_add(controller_reserve_nano))
        .ok_or_else(|| "customer total nanoTON overflow".to_string())?;

    // Defence-in-depth: re-check the integer sum matches
    if customer_total_nano
        != expert_amount_nano + platform_fee_nano + gas_reserve_nano + controller_reserve_nano
    {
        return Err("internal split inconsistency before calling TON Worker".to_string());
    }

    let prepare_payload = TonController::build_create_booking_contract_request(
        &saved_booking,
        &saved_payment,
        &expert,
        expert_amount_nano.to_string(),
        platform_fee_nano.to_string(),
        gas_reserve_nano.to_string(),
        controller_reserve_nano.to_string(),
        "TON".to_string(),
    )?;

    let expert_confirmation_deadline_unix =
        prepare_payload.expert_confirmation_deadline_unix;

    let session_outcome_deadline_unix =
        prepare_payload.session_outcome_deadline_unix;

    let ton_client = TonWorkerClient::new(
        state.config.ton_worker_base_url.clone(),
        state.config.ton_worker_auth_token.clone(),
    );
    let ton_controller = TonController::new(ton_client);

    let contract = ton_controller
        .create_booking_contract(prepare_payload)
        .await?;

    let mut payment_model: payments::ActiveModel = saved_payment.clone().into();

    payment_model.contract_address = Set(Some(contract.contract_address.clone()));
    payment_model.transaction_ref = Set(Some(format!(
        "prepared:{}",
        contract.contract_address
    )));

    payment_model.updated_at = Set(now);

    let expert_amount_ton_string = expert_amount_ton.normalize().to_string();
    let platform_fee_ton_string = platform_fee_ton.normalize().to_string();
    let gas_reserve_ton_string = gas_reserve_ton.normalize().to_string();
    let controller_reserve_ton_string = controller_reserve_ton.normalize().to_string();
    let customer_total_ton_string = customer_total_ton.normalize().to_string();

    let updated_payment = payment_model
        .update(&state.db)
        .await
        .map_err(|e| format!("failed to update payment: {e}"))?;

    let mut metadata = saved_booking.metadata.clone().unwrap_or_else(|| json!({}));

    if !metadata.is_object() {
        metadata = json!({});
    }

    if let Some(object) = metadata.as_object_mut() {
        object.insert(
            "ton_payment".to_string(),
            json!({
                "contract_address": contract.contract_address.clone(),

                "hourly_rate_snapshot_usd": saved_booking.hourly_rate_snapshot,
                "requested_duration_minutes": saved_booking.requested_duration_minutes,
                "amount_quoted_usd": saved_booking.amount_quoted,
                "booking_currency": saved_booking.currency.clone(),

                "expert_amount_ton": expert_amount_ton_string.clone(),
                "platform_fee_ton": platform_fee_ton_string.clone(),
                "gas_reserve_ton": gas_reserve_ton_string.clone(),
                "controller_reserve_ton": controller_reserve_ton_string.clone(),
                "customer_total_ton": customer_total_ton_string.clone(),

                "expert_amount_nano_ton": expert_amount_nano.to_string(),
                "platform_fee_nano_ton": platform_fee_nano.to_string(),
                "gas_reserve_nano_ton": gas_reserve_nano.to_string(),
                "controller_reserve_nano_ton": controller_reserve_nano.to_string(),
                "customer_total_nano_ton": customer_total_nano.to_string(),
                "expert_confirmation_deadline_unix": expert_confirmation_deadline_unix,
                "session_outcome_deadline_unix": session_outcome_deadline_unix,

                "wallet_send_amount_nano_ton": contract.amount_nano_ton.clone(),
                "recommended_gas_buffer_nano_ton": contract.recommended_gas_buffer_nano_ton.clone(),
                "total_deploy_value_nano_ton": contract.total_deploy_value_nano_ton.clone(),
                "state_init_boc": contract.state_init_boc.clone()
            }),
        );
    }

    let mut booking_metadata_model: bookings::ActiveModel = saved_booking.clone().into();
    booking_metadata_model.metadata = Set(Some(metadata));
    booking_metadata_model.updated_at = Set(now);

    booking_metadata_model
        .update(&state.db)
        .await
        .map_err(|e| format!("failed to store TON payment metadata on booking: {e}"))?;

    Ok(BeginPaymentResponse {
        booking_id: saved_booking.id,
        payment_id: updated_payment.id,
        contract_address: contract.contract_address.clone(),
        destination_address: contract.contract_address.clone(),
        state_init_boc: contract.state_init_boc.clone(),
        amount_nano_ton: contract.amount_nano_ton.clone(),
        customer_total_ton: customer_total_ton.normalize().to_string(),
        customer_total_nano_ton: customer_total_nano.to_string(),
        recommended_gas_buffer_nano_ton: contract.recommended_gas_buffer_nano_ton.clone(),
        total_deploy_value_nano_ton: contract.total_deploy_value_nano_ton.clone(),
        payment_status: updated_payment.status,
    })
}

pub async fn confirm_booking_payment(
    state: &AppState,
    booking_id: i64,
    body: ConfirmBookingPaymentRequest,
) -> Result<ConfirmBookingPaymentResponse, String> {
    let boc_len = body
        .boc
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.len());

    let booking = bookings::Entity::find_by_id(booking_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load booking: {e}"))?
        .ok_or_else(|| "booking not found".to_string())?;

    if booking.status != BOOKING_STATUS_AWAITING_PAYMENT
        && booking.status != BOOKING_STATUS_FUNDED
    {
        return Err(format!(
            "booking cannot confirm payment from status {}",
            booking.status
        ));
    }

    let payment = payments::Entity::find()
        .filter(payments::Column::BookingId.eq(booking.id))
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load payment: {e}"))?
        .ok_or_else(|| "payment not found".to_string())?;

    if booking.status == BOOKING_STATUS_FUNDED
        && payment.status == PAYMENT_STATUS_FUNDED
    {
        return Ok(ConfirmBookingPaymentResponse {
            booking_id: booking.id,
            payment_id: payment.id,
            booking_status: booking.status,
            payment_status: payment.status,
            contract_address: payment.contract_address.clone().unwrap(),
            transaction_ref: payment.transaction_ref.clone(),
            verified: true,
            contract_state: None,
            account_state: "already_funded".to_string(),
            balance_nano_ton: "0".to_string(),
        });
    }

    if booking.status == BOOKING_STATUS_WAITING_FOR_SESSION {
        return Ok(ConfirmBookingPaymentResponse {
            booking_id: booking.id,
            payment_id: payment.id,
            booking_status: booking.status,
            payment_status: payment.status,
            contract_address: payment
                .contract_address
                .clone()
                .unwrap_or_default(),
            transaction_ref: payment.transaction_ref.clone(),
            verified: true,
            contract_state: None,
            account_state: "waiting_for_session".to_string(),
            balance_nano_ton: "0".to_string(),
        });
    }

    let contract_address = payment
        .contract_address
        .clone()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "payment contract address is missing".to_string())?;

    let now = Utc::now().fixed_offset();

    let tx_ref = body
        .trace_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| format!("wallet_trace:{v}"))
        .or_else(|| boc_len.map(|len| format!("wallet_boc_len:{len}")))
        .unwrap_or_else(|| "contract_state_verified".to_string());

    let ton_client = TonWorkerClient::new(
        state.config.ton_worker_base_url.clone(),
        state.config.ton_worker_auth_token.clone(),
    );

    let ton_controller = TonController::new(ton_client);

    let mut last_state = None;

    let was_already_funded = booking.status == BOOKING_STATUS_FUNDED
        || payment.status == PAYMENT_STATUS_FUNDED;

    let expert = experts::Entity::find_by_id(booking.expert_id)
        .one(&state.db)
        .await
        .map_err(|e| format!("failed to load expert: {e}"))?
        .ok_or_else(|| "expert not found".to_string())?;

    let ton_payment = TonPaymentMetadata::from_booking(&booking)?;

    let expected_customer_total_nano_ton =
        ton_payment.customer_total_nano_ton;

    for attempt in 0..15 {
        let contract_state = ton_controller
            .get_booking_contract_state(&contract_address)
            .await?;

        if contract_state.is_funded {
            if !was_already_funded {
                if !claim_funded(&state.db, booking.id).await? {
                    // Another request already processed this payment.

                    let booking = bookings::Entity::find_by_id(booking.id)
                        .one(&state.db)
                        .await
                        .map_err(|e| format!("failed to reload booking: {e}"))?
                        .ok_or_else(|| "booking not found".to_string())?;

                    let payment = payments::Entity::find()
                        .filter(payments::Column::BookingId.eq(booking.id))
                        .one(&state.db)
                        .await
                        .map_err(|e| format!("failed to reload payment: {e}"))?
                        .ok_or_else(|| "payment not found".to_string())?;

                    return Ok(ConfirmBookingPaymentResponse {
                        booking_id: booking.id,
                        payment_id: payment.id,
                        booking_status: booking.status,
                        payment_status: payment.status,
                        contract_address: contract_address.clone(),
                        transaction_ref: payment.transaction_ref.clone(),
                        verified: true,
                        contract_state: contract_state.contract_state,
                        account_state: contract_state.account_state.clone(),
                        balance_nano_ton: contract_state.balance_nano_ton.clone(),
                    });
                }
            }

            let contract_amount_nano_ton = contract_state
                .contract_amount_nano_ton
                .as_deref()
                .ok_or_else(|| "contract state is missing contract_amount_nano_ton".to_string())
                .and_then(|value| parse_nano_ton(value, "contract amount"))?;

            if contract_amount_nano_ton != expected_customer_total_nano_ton {
                return Err(format!(
                    "contract amount check failed: contract amount {} nanoTON does not match expected customer total {} nanoTON",
                    contract_amount_nano_ton,
                    expected_customer_total_nano_ton
                ));
            }

            let actual_balance_nano_ton =
                parse_nano_ton(&contract_state.balance_nano_ton, "contract balance")?;

            let minimum_required_balance =
                expected_customer_total_nano_ton * (10_000 - BALANCE_TOLERANCE_BPS) / 10_000;

            if actual_balance_nano_ton < minimum_required_balance {
                return Err(format!(
                    "contract is funded but balance check failed: contract balance {} nanoTON is below minimum allowed {} nanoTON (expected {}, tolerance 0.3%)",
                    actual_balance_nano_ton,
                    minimum_required_balance,
                    expected_customer_total_nano_ton,
                ));
            }

            let expert_response_expires_at = ton_payment.expert_confirmation_deadline;

            let mut booking_model: bookings::ActiveModel = booking.clone().into();
            booking_model.status = Set(BOOKING_STATUS_FUNDED.to_string());
            booking_model.expires_at = Set(expert_response_expires_at);
            booking_model.updated_at = Set(now);

            let mut payment_model: payments::ActiveModel = payment.clone().into();
            payment_model.status = Set(PAYMENT_STATUS_FUNDED.to_string());
            payment_model.updated_at = Set(now);

            let txn = state.db
                .begin()
                .await
                .map_err(|e| format!("failed to begin transaction: {e}"))?;

            let updated_booking = booking_model
                .update(&txn)
                .await
                .map_err(|e| format!("failed to mark booking funded: {e}"))?;

            let updated_payment = payment_model
                .update(&txn)
                .await
                .map_err(|e| format!("failed to mark payment funded: {e}"))?;

            txn.commit()
                .await
                .map_err(|e| format!("failed to commit transaction: {e}"))?;

            if !was_already_funded {
                let bot = TelegramBotClient::new(state.config.active_telegram_bot_token.clone());

                if let Err(err) = bot
                    .send_expert_booking_confirmation_request(
                        &expert,
                        &updated_booking,
                        &updated_payment,
                    )
                    .await
                {
                    eprintln!(
                        "[telegram] failed to send expert booking confirmation request: {err}"
                    );
                }

                if let Err(err) = bot
                    .notify_customer_payment_confirmed(&updated_booking)
                    .await
                {
                    eprintln!(
                        "[telegram] failed to notify customer about payment confirmation: {err}"
                    );
                }
            }

            return Ok(ConfirmBookingPaymentResponse {
                booking_id: updated_booking.id,
                payment_id: updated_payment.id,
                booking_status: updated_booking.status,
                payment_status: updated_payment.status,
                contract_address,
                transaction_ref: updated_payment.transaction_ref,
                verified: true,
                contract_state: contract_state.contract_state,
                account_state: contract_state.account_state,
                balance_nano_ton: contract_state.balance_nano_ton,
            });
        }

        last_state = Some(contract_state);

        if attempt < 14 {
            sleep(TokioDuration::from_secs(2)).await;
        }
    }

    let mut payment_model: payments::ActiveModel = payment.clone().into();
    payment_model.status = Set(PAYMENT_STATUS_SUBMITTED.to_string());
    payment_model.transaction_ref = Set(Some(tx_ref.clone()));
    payment_model.updated_at = Set(now);

    let updated_payment = payment_model
        .update(&state.db)
        .await
        .map_err(|e| format!("failed to store submitted payment: {e}"))?;

    let state_snapshot = last_state
        .ok_or_else(|| "failed to read contract state".to_string())?;

    Ok(ConfirmBookingPaymentResponse {
        booking_id: booking.id,
        payment_id: updated_payment.id,
        booking_status: booking.status,
        payment_status: updated_payment.status,
        contract_address,
        transaction_ref: updated_payment.transaction_ref,
        verified: false,
        contract_state: state_snapshot.contract_state,
        account_state: state_snapshot.account_state,
        balance_nano_ton: state_snapshot.balance_nano_ton,
    })
}

async fn contract_already_in_state(
    ton: &TonController,
    contract_address: &str,
    expected_state: i32,
) -> Result<bool, String> {
    let state = ton.get_booking_contract_state(contract_address).await?;
    Ok(state.contract_state == Some(expected_state))
}

async fn claim_processing(
    db: &DatabaseConnection,
    booking_id: i64,
) -> Result<bool, String> {
    let now = Utc::now().fixed_offset();

    let result = bookings::Entity::update_many()
        .col_expr(
            bookings::Column::Status,
            Expr::value(BOOKING_STATUS_PROCESSING),
        )
        .col_expr(
            bookings::Column::UpdatedAt,
            Expr::value(now),
        )
        .filter(bookings::Column::Id.eq(booking_id))
        .filter(bookings::Column::Status.eq(BOOKING_STATUS_FUNDED))
        .exec(db)
        .await
        .map_err(|e| format!("failed to claim booking: {e}"))?;

    Ok(result.rows_affected == 1)
}

async fn rollback_processing_to_funded(
    db: &DatabaseConnection,
    booking_id: i64,
) -> Result<(), String> {
    bookings::Entity::update_many()
        .col_expr(
            bookings::Column::Status,
            Expr::value(BOOKING_STATUS_FUNDED),
        )
        .filter(bookings::Column::Id.eq(booking_id))
        .filter(bookings::Column::Status.eq(BOOKING_STATUS_PROCESSING))
        .exec(db)
        .await
        .map_err(|e| format!("failed to rollback booking claim: {e}"))?;

    Ok(())
}

async fn claim_funded(
    db: &DatabaseConnection,
    booking_id: i64,
) -> Result<bool, String> {
    let result = bookings::Entity::update_many()
        .col_expr(
            bookings::Column::Status,
            Expr::value(BOOKING_STATUS_FUNDED),
        )
        .filter(bookings::Column::Id.eq(booking_id))
        .filter(bookings::Column::Status.eq(BOOKING_STATUS_AWAITING_PAYMENT))
        .exec(db)
        .await
        .map_err(|e| format!("failed to update bookings: {e}"))?;

    Ok(result.rows_affected == 1)
}