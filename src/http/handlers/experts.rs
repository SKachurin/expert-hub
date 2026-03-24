use actix_web::{post, web, HttpResponse};

use crate::{
    services::experts::{upsert_expert, UpsertExpertRequest},
    state::AppState,
};

#[post("/experts/upsert")]
pub async fn upsert_expert_handler(
    state: web::Data<AppState>,
    body: web::Json<UpsertExpertRequest>,
) -> HttpResponse {
    match upsert_expert(&state.db, body.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": err
        })),
    }
}