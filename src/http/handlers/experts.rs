use actix_web::{get, post, web, HttpResponse};

use crate::{
    services::experts::{
        get_edit_expert_by_slug,
        get_public_expert_by_slug,
        update_expert_profile_by_slug,
        upsert_expert,
        UpdateExpertProfileRequest,
        UpsertExpertRequest,
    },
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

#[get("/api/experts/{slug}/edit")]
pub async fn get_edit_expert_handler(
    state: web::Data<AppState>,
    slug: web::Path<String>,
) -> HttpResponse {
    match get_edit_expert_by_slug(&state.db, slug.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(err) if err == "expert not found" => {
            HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": err
            }))
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": err
        })),
    }
}

#[post("/api/experts/{slug}/edit")]
pub async fn update_edit_expert_handler(
    state: web::Data<AppState>,
    slug: web::Path<String>,
    body: web::Json<UpdateExpertProfileRequest>,
) -> HttpResponse {
    match update_expert_profile_by_slug(&state.db, slug.into_inner(), body.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(err) if err == "expert not found" => {
            HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": err
            }))
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": err
        })),
    }
}