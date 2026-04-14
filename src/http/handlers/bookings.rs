use actix_web::{post, web, HttpResponse};

use crate::{
    services::bookings::{
        begin_booking_payment,
        create_booking_request,
        BeginPaymentRequest,
        CreateBookingRequest,
    },
    state::AppState,
};

#[post("/api/bookings/request")]
pub async fn create_booking_request_handler(
    state: web::Data<AppState>,
    body: web::Json<CreateBookingRequest>,
) -> HttpResponse {
    match create_booking_request(&state, body.into_inner()).await {
        Ok(saved) => HttpResponse::Ok().json(saved),
        Err(message) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": message
        })),
    }
}

#[post("/api/bookings/{booking_id}/begin-payment")]
pub async fn begin_booking_payment_handler(
    state: web::Data<AppState>,
    booking_id: web::Path<i64>,
    body: web::Json<BeginPaymentRequest>,
) -> HttpResponse {
    match begin_booking_payment(&state, booking_id.into_inner(), body.into_inner()).await {
        Ok(saved) => HttpResponse::Ok().json(saved),
        Err(message) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": message
        })),
    }
}   