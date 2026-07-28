use actix_web::web;
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::time::{interval, Duration as TokioDuration};

use crate::{
    entities::{bookings, payments},
    services::bookings::{
        expire_unanswered_funded_booking,
        BOOKING_STATUS_FUNDED,
        PAYMENT_STATUS_FUNDED,
    },
    state::AppState,
};

pub async fn run_booking_expiry_worker(state: web::Data<AppState>) {
    let mut ticker = interval(TokioDuration::from_secs(60));

    loop {
        ticker.tick().await;

        if let Err(err) = expire_due_bookings_once(&state).await {
            eprintln!("[booking-expiry-worker] {err}");
        }
    }
}

async fn expire_due_bookings_once(state: &web::Data<AppState>) -> Result<(), String> {
    let now = Utc::now().fixed_offset();

    let due_bookings = bookings::Entity::find()
        .filter(bookings::Column::Status.eq(BOOKING_STATUS_FUNDED))
        .filter(bookings::Column::ExpiresAt.lte(now))
        .all(&state.db)
        .await
        .map_err(|e| format!("failed to load expired funded bookings: {e}"))?;

    for booking in due_bookings {
        let Some(payment) = payments::Entity::find()
            .filter(payments::Column::BookingId.eq(booking.id))
            .filter(payments::Column::Status.eq(PAYMENT_STATUS_FUNDED))
            .one(&state.db)
            .await
            .map_err(|e| {
                format!(
                    "failed to load funded payment for booking {}: {e}",
                    booking.id
                )
            })?
        else {
            continue;
        };

        if let Err(err) =
            expire_unanswered_funded_booking(state.get_ref(), &booking, &payment).await
        {
            eprintln!(
                "[booking-expiry-worker] failed to expire booking {}: {}",
                booking.id, err
            );
        }
    }

    Ok(())
}