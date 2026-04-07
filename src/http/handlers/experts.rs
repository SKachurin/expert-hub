use actix_web::{delete, get, post, web, HttpResponse};
use chrono::{Duration, Utc};

use crate::{
    services::{
        calendar_connections::{
            delete_calendar_connection_for_expert,
            upsert_calendar_connection,
            UpsertCalendarConnectionData,
        },
        experts::{
            get_edit_expert_by_slug,
            update_expert_profile_by_slug,
            upsert_expert,
            UpdateExpertProfileRequest,
            UpsertExpertRequest,
        },
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
    let slug = slug.into_inner();
    let payload = body.into_inner();

    let attach_google_session_ids = payload.attach_google_session_ids.clone();

    let updated = match update_expert_profile_by_slug(&state.db, slug.clone(), payload).await {
        Ok(response) => response,
        Err(err) if err == "expert not found" => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": err
            }))
        }
        Err(err) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": err
            }))
        }
    };

    let mut has_primary_already = updated.calendar_connections.iter().any(|c| c.is_primary);

    for session_id in attach_google_session_ids {
        let session = match state.google_oauth_sessions.lock() {
            Ok(guard) => guard.get(&session_id).cloned(),
            Err(_) => None,
        };

        let Some(session) = session else {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": format!("google session not found: {}", session_id)
            }));
        };

        if session.selected_calendar_ids.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": format!("no calendars selected in google session: {}", session_id)
            }));
        }

        let token_expires_at = session
            .expires_in
            .map(|seconds| Utc::now().fixed_offset() + Duration::seconds(seconds));

        let scopes_json = session.scopes.as_deref().map(|scopes| {
            let items: Vec<String> = scopes
                .split(|c: char| c == ' ' || c == ',')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(ToOwned::to_owned)
                .collect();

            serde_json::json!(items)
        });

        for selected_id in &session.selected_calendar_ids {
            let Some(selected_calendar) = session.calendars.iter().find(|c| c.id == *selected_id) else {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "error",
                    "message": format!("selected google calendar not found in session: {}", selected_id)
                }));
            };

            let connection_label = match session.account_email.as_deref() {
                Some(email) if !email.trim().is_empty() => {
                    format!("Google · {} · {}", email.trim(), selected_calendar.summary)
                }
                _ => format!("Google · {}", selected_calendar.summary),
            };

            let provider_metadata = serde_json::json!({
                "oauth_session_id": session.session_id,
                "calendar_access_role": selected_calendar.access_role,
                "calendar_primary": selected_calendar.primary,
                "session_created_at_unix": session.created_at_unix,
            });

            let is_first_calendar = !has_primary_already;

            let save_result = upsert_calendar_connection(
                &state.db,
                UpsertCalendarConnectionData {
                    expert_id: updated.id,
                    provider: "google".to_string(),
                    connection_label: Some(connection_label),
                    is_primary: is_first_calendar,
                    is_enabled: true,
                    connection_status: Some("connected".to_string()),
                    account_email: session.account_email.clone(),
                    provider_account_id: session.provider_account_id.clone(),
                    provider_user_uri: None,
                    provider_organization_uri: None,
                    selected_calendar_id: Some(selected_calendar.id.clone()),
                    selected_calendar_name: Some(selected_calendar.summary.clone()),
                    selected_calendar_timezone: selected_calendar.time_zone.clone(),
                    selected_event_type_uri: None,
                    selected_event_type_name: None,
                    selected_scheduling_url: None,
                    access_token: Some(session.access_token.clone()),
                    refresh_token: session.refresh_token.clone(),
                    token_expires_at,
                    scopes_json: scopes_json.clone(),
                    provider_metadata: Some(provider_metadata),
                    sync_cursor: None,
                    last_sync_at: None,
                    last_sync_status: None,
                    last_sync_error: None,
                    webhook_signing_secret: None,
                    public_link: None,
                },
            )
            .await;

            if let Err(err) = save_result {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "error",
                    "message": err
                }));
            }

            if is_first_calendar {
                has_primary_already = true;
            }
        }
    }

    match get_edit_expert_by_slug(&state.db, slug).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": err
        })),
    }
}

#[delete("/api/experts/{slug}/calendar-connections/{connection_id}")]
pub async fn delete_calendar_connection_handler(
    state: web::Data<AppState>,
    path: web::Path<(String, i64)>,
) -> HttpResponse {
    let (slug, connection_id) = path.into_inner();

    let expert = match get_edit_expert_by_slug(&state.db, slug).await {
        Ok(v) => v,
        Err(err) if err == "expert not found" => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": err
            }))
        }
        Err(err) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": err
            }))
        }
    };

    match delete_calendar_connection_for_expert(&state.db, expert.id, connection_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ok"
        })),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": err
        })),
    }
}