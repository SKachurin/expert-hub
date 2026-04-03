use actix_web::{post, web, HttpResponse};

use crate::services::expert_setup::{register_expert_setup, RegisterExpertSetupRequest};
use crate::state::AppState;

#[post("/expert-setup/register")]
pub async fn register_expert_setup_handler(
    state: web::Data<AppState>,
    body: web::Json<RegisterExpertSetupRequest>,
) -> HttpResponse {
    match register_expert_setup(&state, body.into_inner()).await {
        Ok(saved) => HttpResponse::Ok().json(saved),
        Err(message) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": message
        })),
    }
}