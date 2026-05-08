use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde_json::json;
use std::collections::HashMap;

use crate::entities::{bookings, calendar_connections, experts, payments};
use crate::services::telegram_bot::TelegramBotClient;
use crate::state::AppState;

use super::deletion_refunds::process_deleted_expert_refunds;
use super::dto::{
    DeleteExpertPaidBookingPreview,
    DeleteExpertPreviewRequest,
    DeleteExpertPreviewResponse,
    DeleteExpertRequest,
    DeleteExpertResponse,
};

const BOOKING_STATUS_FUNDED: &str = "funded";
const BOOKING_STATUS_WAITING_FOR_SESSION: &str = "waiting_for_session";
const BOOKING_STATUS_IN_GRACE_PERIOD: &str = "in_grace_period";

const PAYMENT_STATUS_FUNDED: &str = "funded";

pub async fn preview_delete_expert_by_slug<C>(
    db: &C,
    slug: String,
    body: DeleteExpertPreviewRequest,
) -> Result<DeleteExpertPreviewResponse, String>
where
    C: ConnectionTrait,
{
    let expert = load_owned_expert(db, slug, body.telegram_id).await?;
    let paid_future_bookings = load_paid_future_bookings(db, expert.id).await?;

    Ok(DeleteExpertPreviewResponse {
        has_paid_future_bookings: !paid_future_bookings.is_empty(),
        paid_future_bookings,
    })
}

pub async fn delete_expert_profile_by_slug(
    state: &AppState,
    slug: String,
    body: DeleteExpertRequest,
) -> Result<DeleteExpertResponse, String> {
    let expert = load_owned_expert(&state.db, slug, body.telegram_id).await?;
    let paid_future_bookings = load_paid_future_bookings(&state.db, expert.id).await?;

    if !paid_future_bookings.is_empty() && !body.confirm_paid_future_bookings {
        return Err("paid future bookings require explicit confirmation".to_string());
    }

    erase_expert_profile_data(&state.db, &expert).await?;

    if paid_future_bookings.is_empty() {
        let bot = TelegramBotClient::new(state.config.active_telegram_bot_token.clone());

        if let Err(err) = bot
            .notify_expert_profile_deleted_no_paid_bookings(expert.telegram_id)
            .await
        {
            eprintln!("[telegram] failed to notify deleted expert: {err}");
        }

        return Ok(DeleteExpertResponse {
            deleted: true,
            refunds_dispatched: false,
            paid_future_bookings_count: 0,
            redirect_to: "/".to_string(),
        });
    }

    let db_for_job = state.db.clone();
    let bot_token_for_job = state.config.active_telegram_bot_token.clone();
    let ton_worker_base_url_for_job = state.config.ton_worker_base_url.clone();
    let ton_worker_auth_token_for_job = state.config.ton_worker_auth_token.clone();

    let bookings_for_job = paid_future_bookings.clone();
    let expert_telegram_id = expert.telegram_id;

    actix_web::rt::spawn(async move {
        process_deleted_expert_refunds(
            db_for_job,
            bot_token_for_job,
            ton_worker_base_url_for_job,
            ton_worker_auth_token_for_job,
            bookings_for_job,
            expert_telegram_id,
        )
        .await;
    });

    Ok(DeleteExpertResponse {
        deleted: true,
        refunds_dispatched: true,
        paid_future_bookings_count: paid_future_bookings.len(),
        redirect_to: "/".to_string(),
    })
}

async fn load_owned_expert<C>(
    db: &C,
    slug: String,
    telegram_id: i64,
) -> Result<experts::Model, String>
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

    if expert.telegram_id != telegram_id {
        return Err("you are not allowed to delete this profile".to_string());
    }

    Ok(expert)
}

async fn load_paid_future_bookings<C>(
    db: &C,
    expert_id: i64,
) -> Result<Vec<DeleteExpertPaidBookingPreview>, String>
where
    C: ConnectionTrait,
{
    let now = Utc::now().fixed_offset();

    let future_bookings = bookings::Entity::find()
        .filter(bookings::Column::ExpertId.eq(expert_id))
        .filter(bookings::Column::SlotStart.gt(now))
        .filter(bookings::Column::Status.is_in([
            BOOKING_STATUS_FUNDED,
            BOOKING_STATUS_WAITING_FOR_SESSION,
            BOOKING_STATUS_IN_GRACE_PERIOD,
        ]))
        .order_by_asc(bookings::Column::SlotStart)
        .all(db)
        .await
        .map_err(|e| format!("failed to query future bookings: {e}"))?;

    if future_bookings.is_empty() {
        return Ok(Vec::new());
    }

    let booking_ids = future_bookings
        .iter()
        .map(|booking| booking.id)
        .collect::<Vec<_>>();

    let funded_payments = payments::Entity::find()
        .filter(payments::Column::BookingId.is_in(booking_ids))
        .filter(payments::Column::Status.eq(PAYMENT_STATUS_FUNDED))
        .all(db)
        .await
        .map_err(|e| format!("failed to query funded payments: {e}"))?;

    let payments_by_booking_id = funded_payments
        .into_iter()
        .map(|payment| (payment.booking_id, payment))
        .collect::<HashMap<_, _>>();

    let result = future_bookings
        .into_iter()
        .filter_map(|booking| {
            let payment = payments_by_booking_id.get(&booking.id)?;

            Some(DeleteExpertPaidBookingPreview {
                booking_id: booking.id,
                payment_id: payment.id,
                customer_telegram_id: booking.requested_by_telegram_id,
                customer_username: booking.requested_by_username.clone(),
                customer_display_name: booking.requested_by_display_name.clone(),
                slot_start: booking.slot_start.to_rfc3339(),
                slot_end: booking.slot_end.to_rfc3339(),
                duration_minutes: booking.requested_duration_minutes,
                amount_quoted: booking.amount_quoted.to_string(),
                currency: booking.currency.clone(),
                booking_status: booking.status.clone(),
                payment_status: payment.status.clone(),
                contract_address: payment.contract_address.clone(),
            })
        })
        .collect();

    Ok(result)
}

async fn erase_expert_profile_data<C>(
    db: &C,
    expert: &experts::Model,
) -> Result<(), String>
where
    C: ConnectionTrait,
{
    let now = Utc::now().fixed_offset();
    let deleted_suffix = format!("{}-{}", expert.id, now.timestamp());

    let mut active: experts::ActiveModel = expert.clone().into();

    active.first_name = Set("Deleted".to_string());
    active.last_name = Set(None);
    active.username = Set(None);
    active.display_name = Set("Deleted expert".to_string());
    active.telegram_bio = Set(None);
    active.photo_url = Set(None);

    active.ton_wallet_address = Set(format!("deleted:{}:{}", expert.id, now.timestamp()));
    active.public_slug = Set(format!("deleted-{}", deleted_suffix));

    active.is_active = Set(false);
    active.is_bookable = Set(false);

    active.working_days = Set(json!([]));
    active.allowed_session_durations = Set(json!([]));

    active.updated_at = Set(now);

    active
        .update(db)
        .await
        .map_err(|e| format!("failed to erase expert profile data: {e}"))?;

    let connections = calendar_connections::Entity::find()
        .filter(calendar_connections::Column::ExpertId.eq(expert.id))
        .all(db)
        .await
        .map_err(|e| format!("failed to query calendar connections: {e}"))?;

    for connection in connections {
        let mut active_connection: calendar_connections::ActiveModel = connection.into();

        active_connection.is_enabled = Set(false);
        active_connection.connection_status = Set("deleted".to_string());
        active_connection.access_token = Set(None);
        active_connection.refresh_token = Set(None);
        active_connection.token_expires_at = Set(None);
        active_connection.updated_at = Set(now);

        active_connection
            .update(db)
            .await
            .map_err(|e| format!("failed to erase calendar connection data: {e}"))?;
    }

    Ok(())
}