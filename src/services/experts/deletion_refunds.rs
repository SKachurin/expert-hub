use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::entities::{bookings, payments};
use crate::services::telegram_bot::TelegramBotClient;
use crate::services::ton::{
    client::TonWorkerClient,
    controller::TonController,
};

use super::dto::DeleteExpertPaidBookingPreview;

const BOOKING_STATUS_EXPERT_NO_SHOW: &str = "expert_no_show";
const BOOKING_STATUS_REFUNDED: &str = "refunded";

const PAYMENT_STATUS_REFUNDED: &str = "refunded";

pub async fn process_deleted_expert_refunds(
    db: DatabaseConnection,
    bot_token: String,
    ton_worker_base_url: String,
    ton_worker_auth_token: String,
    paid_future_bookings: Vec<DeleteExpertPaidBookingPreview>,
    expert_telegram_id: i64,
) {
    let bot = TelegramBotClient::new(bot_token);
    let ton_controller = TonController::new(TonWorkerClient::new(
        ton_worker_base_url,
        ton_worker_auth_token,
    ));

    for item in paid_future_bookings {
        let Some(contract_address) = item.contract_address.clone() else {
            eprintln!(
                "[delete-profile] booking {} has funded payment but no contract address",
                item.booking_id
            );
            continue;
        };

        let refund_result = ton_controller
            .settle_expert_no_show(contract_address, item.payment_id, item.booking_id)
            .await;

        match refund_result {
            Ok(_) => {
                let now = Utc::now().fixed_offset();

                if let Ok(Some(booking)) = bookings::Entity::find_by_id(item.booking_id)
                    .one(&db)
                    .await
                {
                    let mut active_booking: bookings::ActiveModel = booking.into();
                    active_booking.status = Set(BOOKING_STATUS_REFUNDED.to_string());
                    active_booking.outcome_source = Set(Some("expert_profile_deleted".to_string()));
                    active_booking.updated_at = Set(now);

                    if let Err(err) = active_booking.update(&db).await {
                        eprintln!(
                            "[delete-profile] failed to mark booking {} refunded: {err}",
                            item.booking_id
                        );
                    }
                }

                if let Ok(Some(payment)) = payments::Entity::find_by_id(item.payment_id)
                    .one(&db)
                    .await
                {
                    let mut active_payment: payments::ActiveModel = payment.into();
                    active_payment.status = Set(PAYMENT_STATUS_REFUNDED.to_string());
                    active_payment.updated_at = Set(now);

                    if let Err(err) = active_payment.update(&db).await {
                        eprintln!(
                            "[delete-profile] failed to mark payment {} refunded: {err}",
                            item.payment_id
                        );
                    }
                }

                if let Err(err) = bot
                    .notify_customer_expert_deleted_refund_started(
                        item.customer_telegram_id,
                        item.booking_id,
                    )
                    .await
                {
                    eprintln!(
                        "[telegram] failed to notify customer {} about deleted expert refund: {err}",
                        item.customer_telegram_id
                    );
                }
            }

            Err(err) => {
                eprintln!(
                    "[delete-profile] failed to process expert_no_show refund for booking {} payment {}: {err}",
                    item.booking_id,
                    item.payment_id
                );

                if let Ok(Some(booking)) = bookings::Entity::find_by_id(item.booking_id)
                    .one(&db)
                    .await
                {
                    let now = Utc::now().fixed_offset();

                    let mut active_booking: bookings::ActiveModel = booking.into();
                    active_booking.status = Set(BOOKING_STATUS_EXPERT_NO_SHOW.to_string());
                    active_booking.outcome_source =
                        Set(Some("expert_profile_deleted_refund_failed".to_string()));
                    active_booking.updated_at = Set(now);

                    let _ = active_booking.update(&db).await;
                }
            }
        }
    }

    if let Err(err) = bot
        .notify_expert_profile_deleted_refunds_started(expert_telegram_id)
        .await
    {
        eprintln!("[telegram] failed to notify deleted expert: {err}");
    }
}