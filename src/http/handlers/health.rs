use actix_web::{get, HttpResponse};
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    timestamp_utc: String,
}

#[get("/health")]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        service: "expert-hub",
        version: env!("CARGO_PKG_VERSION"),
        timestamp_utc: Utc::now().to_rfc3339(),
    })
}