use actix_web::{post, web, HttpResponse};

use crate::{
    services::expert_setup::{register_expert_setup, RegisterExpertSetupRequest},
    state::AppState,
};

#[post("/expert-setup/register")]
pub async fn register_expert_setup_handler(
    state: web::Data<AppState>,
    body: web::Json<RegisterExpertSetupRequest>,
) -> HttpResponse {
    match register_expert_setup(&state.db, body.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": err
        })),
    }
}