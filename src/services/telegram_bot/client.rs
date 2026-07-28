use chrono::{DateTime, FixedOffset};

use crate::entities::{bookings, experts, payments};

use super::dto::{
    AnswerCallbackQueryRequest,
    EditMessageReplyMarkupRequest,
    InlineKeyboardButton,
    InlineKeyboardMarkup,
    SendMessageRequest,
};

#[derive(Clone)]
pub struct TelegramBotClient {
    token: String,
    http: reqwest::Client,
}

impl TelegramBotClient {
    pub fn new(token: String) -> Self {
        Self {
            token,
            http: reqwest::Client::new(),
        }
    }

    fn api_url(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.token, method)
    }

    pub async fn send_message(
        &self,
        chat_id: i64,
        text: String,
        reply_markup: Option<InlineKeyboardMarkup>,
    ) -> Result<(), String> {
        let payload = SendMessageRequest {
            chat_id,
            text,
            reply_markup,
        };

        let response = self
            .http
            .post(self.api_url("sendMessage"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("telegram sendMessage failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("telegram sendMessage returned {status}: {text}"));
        }

        Ok(())
    }

    pub async fn answer_callback_query(
        &self,
        callback_query_id: String,
        text: Option<String>,
        show_alert: bool,
    ) -> Result<(), String> {
        let payload = AnswerCallbackQueryRequest {
            callback_query_id,
            text,
            show_alert,
        };

        let response = self
            .http
            .post(self.api_url("answerCallbackQuery"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("telegram answerCallbackQuery failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("telegram answerCallbackQuery returned {status}: {text}"));
        }

        Ok(())
    }

    pub async fn clear_inline_keyboard(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> Result<(), String> {
        let payload = EditMessageReplyMarkupRequest {
            chat_id,
            message_id,
            reply_markup: None,
        };

        let response = self
            .http
            .post(self.api_url("editMessageReplyMarkup"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("telegram editMessageReplyMarkup failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("telegram editMessageReplyMarkup returned {status}: {text}"));
        }

        Ok(())
    }

    pub async fn send_expert_booking_confirmation_request(
        &self,
        expert: &experts::Model,
        booking: &bookings::Model,
        payment: &payments::Model,
    ) -> Result<(), String> {
        let customer_name = booking
            .requested_by_display_name
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or("Customer");

        let customer_username = booking
            .requested_by_username
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .map(|v| format!("@{}", v.trim_start_matches('@')))
            .unwrap_or_else(|| "no username".to_string());

        let slot_start = format_booking_time(&booking.slot_start);
        let slot_end = format_booking_time(&booking.slot_end);
        let confirmation_time_left = format_time_left_until(booking.expires_at);

        let text = format!(
            "New booking request #{}:\n\n\
             Customer: {customer_username} / {customer_name}\n\
             Date: {}\n\
             Time: {}–{}\n\
             Duration: {} minutes\n\
             Amount: {} USD\n\n\
             The customer has funded the escrow contract for this slot.\n\n\
             Please confirm or decline this booking within {confirmation_time_left}.\n\
             If you do not answer in time, the booking will be automatically declined and the escrow refund will be triggered.\n\n\
             Do you confirm this booking?",
             booking.id,
            format_booking_date(&booking.slot_start),
            slot_start,
            slot_end,
            booking.requested_duration_minutes,
            payment.amount,
        );

        let reply_markup = InlineKeyboardMarkup {
            inline_keyboard: vec![vec![
                InlineKeyboardButton {
                    text: "✅ Confirm booking".to_string(),
                    callback_data: format!("booking_confirm:{}:{}", booking.id, payment.id),
                },
                InlineKeyboardButton {
                    text: "❌ Decline booking".to_string(),
                    callback_data: format!("booking_decline:{}:{}", booking.id, payment.id),
                },
            ]],
        };

        fn format_time_left_until(deadline: chrono::DateTime<chrono::FixedOffset>) -> String {
            let now = chrono::Utc::now().fixed_offset();

            if deadline <= now {
                return "less than 1 minute".to_string();
            }

            let minutes = (deadline - now).num_minutes();

            if minutes >= 60 {
                let hours = minutes / 60;
                let rest_minutes = minutes % 60;

                if rest_minutes == 0 {
                    format!("{hours} hour{}", if hours == 1 { "" } else { "s" })
                } else {
                    format!(
                        "{hours} hour{} {rest_minutes} minute{}",
                        if hours == 1 { "" } else { "s" },
                        if rest_minutes == 1 { "" } else { "s" }
                    )
                }
            } else {
                format!("{minutes} minute{}", if minutes == 1 { "" } else { "s" })
            }
        }

        self.send_message(expert.telegram_id, text, Some(reply_markup)).await
    }

    pub async fn notify_customer_payment_confirmed(
        &self,
        booking: &bookings::Model,
    ) -> Result<(), String> {
        self.send_message(
            booking.requested_by_telegram_id,
            format!(
                "Payment confirmed for booking #{}.\n\n\
                 The expert has been notified and will confirm or decline your booking.",
                booking.id
            )
                .to_string(),
            None,
        )
        .await
    }

    pub async fn notify_customer_expert_confirmed(
        &self,
        expert: &experts::Model,
        booking: &bookings::Model,
    ) -> Result<(), String> {
        let expert_username = expert
            .username
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .map(|v| format!("@{}", v.trim_start_matches('@')))
            .unwrap_or_else(|| expert.display_name.clone());

        let text = format!(
            "Your booking #{} is confirmed.\n\n\
             Expert: {expert_username}\n\
             Date: {}\n\
             Time: {}–{}\n\
             Duration: {} minutes\n\n\
             We’ll notify you before the session starts.",
             booking.id,
            format_booking_date(&booking.slot_start),
            format_booking_time(&booking.slot_start),
            format_booking_time(&booking.slot_end),
            booking.requested_duration_minutes,
        );

        self.send_message(booking.requested_by_telegram_id, text, None).await
    }

    pub async fn notify_customer_expert_declined(
        &self,
        booking: &bookings::Model,
    ) -> Result<(), String> {
        self.send_message(
            booking.requested_by_telegram_id,
            format!(
                "The expert declined your booking #{}.\n\nThe escrow refund has been triggered.",
                booking.id
            )
                .to_string(),
            None,
        )
        .await
    }

    pub async fn notify_expert_profile_deleted_no_paid_bookings(
        &self,
        expert_telegram_id: i64,
    ) -> Result<(), String> {
        self.send_message(
            expert_telegram_id,
            "Your Expert Hub profile has been deleted.\n\nYour public page is no longer visible and your profile data is being erased where possible."
                .to_string(),
            None,
        )
        .await
    }

    pub async fn notify_expert_profile_deleted_refunds_started(
        &self,
        expert_telegram_id: i64,
    ) -> Result<(), String> {
        self.send_message(
            expert_telegram_id,
            "Your Expert Hub profile has been deleted.\n\nYour public page is no longer visible. Future paid bookings, if any, have been sent to refund processing."
                .to_string(),
            None,
        )
        .await
    }

    pub async fn notify_customer_expert_deleted_refund_started(
        &self,
        customer_telegram_id: i64,
        booking_id: i64,
    ) -> Result<(), String> {
        let text = format!(
            "The expert has deleted their Expert Hub profile.\n\nYour upcoming booking #{} has been cancelled. The funded escrow contract refund has been started.\n\nSorry for the inconvenience.",
            booking_id
        );

        self.send_message(customer_telegram_id, text, None).await
    }

    pub async fn notify_customer_expert_response_timeout_refunded(
        &self,
        booking: &bookings::Model,
    ) -> Result<(), String> {
        let text = format!(
                       "The expert did not answer in time.\n\n\
                        Booking #{}\n\
                        Your escrow refund has been triggered.",
                       booking.id
                   );

        self.send_message(
            booking.requested_by_telegram_id,
            text.to_string(),
            None,
        )
        .await
    }

    pub async fn notify_expert_response_timeout_refunded(
        &self,
        expert: &experts::Model,
        booking: &bookings::Model,
    ) -> Result<(), String> {
        let text = format!(
                       "You did not answer in time.\n\n\
                        Booking #{}\n\
                        The escrow refund has been triggered.",
                       booking.id
                   );

        self.send_message(
            expert.telegram_id,
            text.to_string(),
            None,
        )
        .await
    }
}

fn format_booking_date(value: &DateTime<FixedOffset>) -> String {
    value.format("%-d %B %Y").to_string()
}

fn format_booking_time(value: &DateTime<FixedOffset>) -> String {
    value.format("%H:%M").to_string()
}