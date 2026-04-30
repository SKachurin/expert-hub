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

        let text = format!(
            "New booking request:\n\n\
             Customer: {customer_username} / {customer_name}\n\
             Date: {}\n\
             Time: {}–{}\n\
             Duration: {} minutes\n\
             Amount: {} TON\n\n\
             The customer has funded the escrow contract for this slot.\n\n\
             Do you confirm this booking?",
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

        self.send_message(expert.telegram_id, text, Some(reply_markup)).await
    }

    pub async fn notify_customer_payment_confirmed(
        &self,
        booking: &bookings::Model,
    ) -> Result<(), String> {
        self.send_message(
            booking.requested_by_telegram_id,
            "Payment confirmed.\n\nThe expert has been notified and will confirm or decline your booking."
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
            "Your booking is confirmed.\n\n\
             Expert: {expert_username}\n\
             Date: {}\n\
             Time: {}–{}\n\
             Duration: {} minutes\n\n\
             We’ll notify you before the session starts.",
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
            "The expert declined this booking.\n\nThe escrow refund has been triggered."
                .to_string(),
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