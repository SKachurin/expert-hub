use actix_web::{post, web, HttpRequest, HttpResponse};

use crate::{
    services::{
        bookings::{
            process_expert_booking_decision,
            ExpertBookingDecision,
        },
        telegram_bot::{TelegramBotClient, TelegramUpdate},
    },
    state::AppState,
};

#[post("/telegram/webhook")]
pub async fn telegram_webhook_handler(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<TelegramUpdate>,
) -> HttpResponse {
//     return HttpResponse::ImATeapot().body("SERGE_TEST");

    let expected_secret = state.config.telegram_webhook_secret_token.trim();

    println!(
        "EXPECTED='{}'",
        expected_secret
    );

    println!(
        "HEADERS={:#?}",
        req.headers()
    );

//     return HttpResponse::Ok().finish();



    if !expected_secret.is_empty() {
        println!("WEBHOOK_HEADERS {:?}", req.headers());

        let incoming_secret = req
            .headers()
            .get("X-Telegram-Bot-Api-Secret-Token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        println!(
            "WEBHOOK_SECRET expected='{}' incoming='{}'",
            expected_secret,
            incoming_secret
        );

        if incoming_secret != expected_secret {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "ok": false,
                "error": "unauthorized"
            }));
        }
    }

    let Some(callback) = &body.callback_query else {
        println!("BOOKING_WEBHOOK received non-callback update");
        return HttpResponse::Ok().json(serde_json::json!({ "ok": true }));
    };

    let Some(data) = callback.data.as_deref() else {
        return HttpResponse::Ok().json(serde_json::json!({ "ok": true }));
    };

    println!(
        "BOOKING_DECLINE_WEBHOOK callback_id={} from={} data={:?}",
        callback.id,
        callback.from.id,
        callback.data
    );

    let parsed = parse_booking_callback_data(data);

    let bot = TelegramBotClient::new(state.config.active_telegram_bot_token.clone());

    let Some((decision, booking_id, payment_id)) = parsed else {
        let _ = bot
            .answer_callback_query(
                callback.id.clone(),
                Some("Unknown action.".to_string()),
                false,
            )
            .await;

        return HttpResponse::Ok().json(serde_json::json!({ "ok": true }));
    };

    println!(
        "BOOKING_CALLBACK decision={:?} booking_id={} payment_id={}",
        decision,
        booking_id,
        payment_id
    );


    let result = process_expert_booking_decision(
        &state,
        booking_id,
        payment_id,
        callback.from.id,
        decision,
    )
    .await;

    match result {
        Ok(_) => {
            let answer_text = match decision {
                ExpertBookingDecision::Confirm => "Booking confirmed.",
                ExpertBookingDecision::Decline => "Booking declined. Refund triggered.",
            };

            let _ = bot
                .answer_callback_query(
                    callback.id.clone(),
                    Some(answer_text.to_string()),
                    false,
                )
                .await;

            if let Some(message) = &callback.message {
                let _ = bot
                    .clear_inline_keyboard(message.chat.id, message.message_id)
                    .await;
            }

            HttpResponse::Ok().json(serde_json::json!({ "ok": true }))
        }

        Err(err) => {
            let _ = bot
                .answer_callback_query(callback.id.clone(), Some(err.clone()), true)
                .await;

            HttpResponse::Ok().json(serde_json::json!({
                "ok": false,
                "error": err
            }))
        }
    }
}

fn parse_booking_callback_data(
    data: &str,
) -> Option<(ExpertBookingDecision, i64, i64)> {
    let mut parts = data.split(':');

    let action = parts.next()?;
    let booking_id = parts.next()?.parse::<i64>().ok()?;
    let payment_id = parts.next()?.parse::<i64>().ok()?;

    if parts.next().is_some() {
        return None;
    }

    match action {
        "booking_confirm" => Some((ExpertBookingDecision::Confirm, booking_id, payment_id)),
        "booking_decline" => Some((ExpertBookingDecision::Decline, booking_id, payment_id)),
        _ => None,
    }
}