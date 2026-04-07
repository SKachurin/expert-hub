use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::google_calendar::{
    build_google_oauth_url, exchange_code_for_tokens, fetch_google_calendars,
    fetch_google_userinfo, has_google_calendar_scope,
};
use crate::state::{AppState, GoogleOAuthSession};

#[derive(Deserialize)]
pub struct GoogleCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct GoogleSelectRequest {
    pub selected_calendar_ids: Vec<String>,
}


#[derive(Deserialize)]
pub struct GoogleStartQuery {
    pub telegram_id: i64,
    #[serde(default)]
    pub return_to: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
}

#[derive(Serialize)]
pub struct GoogleSessionView {
    pub session_id: String,
    pub account_email: Option<String>,
    pub calendars: Vec<crate::state::GoogleCalendarCandidate>,
    pub selected_calendar_ids: Vec<String>,
}

#[derive(Deserialize, Serialize, Default)]
struct GoogleOAuthState {
    telegram_id: i64,
    #[serde(default)]
    return_to: Option<String>,
    #[serde(default)]
    slug: Option<String>,
}

#[get("/oauth/google/start")]
pub async fn google_start(
    state: web::Data<AppState>,
    query: web::Query<GoogleStartQuery>,
) -> impl Responder {
    if query.telegram_id <= 0 {
        return HttpResponse::BadRequest().body("telegram_id is required");
    }

    let state_value = serde_json::to_string(&GoogleOAuthState {
        telegram_id: query.telegram_id,
        return_to: query.return_to.clone(),
        slug: query.slug.clone(),
    })
    .unwrap_or_else(|_| "{\"telegram_id\":0}".to_string());
    let url = build_google_oauth_url(&state.config, &state_value);

    HttpResponse::Found()
        .append_header(("Location", url))
        .finish()
}

#[get("/oauth/google/callback")]
pub async fn google_callback(
    state: web::Data<AppState>,
    query: web::Query<GoogleCallbackQuery>,
) -> impl Responder {
    let oauth_state = parse_google_state(query.state.as_ref());
    let redirect_base = build_google_return_url(&oauth_state);
    if let Some(err) = &query.error {
        let redirect_url = format!(
            "{}?google_error=oauth_error&google_error_message={}",
            redirect_base,
            urlencoding::encode(err),
        );

        return HttpResponse::Found()
            .append_header(("Location", redirect_url))
            .finish();
    }

    let code = match &query.code {
        Some(v) if !v.trim().is_empty() => v.trim(),
        _ => return HttpResponse::BadRequest().body("missing code"),
    };

    let token = match exchange_code_for_tokens(&state.config, code).await {
        Ok(v) => v,
        Err(e) => {
            let redirect_url = format!(
                "{}?google_error=token_exchange_failed&google_error_message={}",
                redirect_base,
                urlencoding::encode(&e),
            );

            return HttpResponse::Found()
                .append_header(("Location", redirect_url))
                .finish();
        }
    };

    let granted_scopes = token.scope.clone().unwrap_or_default();

    if !has_google_calendar_scope(Some(&granted_scopes)) {
        let redirect_url = format!(
            "{}?google_error=calendar_scope_denied&google_scope={}",
            redirect_base,
            urlencoding::encode(&granted_scopes),
        );

        return HttpResponse::Found()
            .append_header(("Location", redirect_url))
            .finish();
    }

    let userinfo = match fetch_google_userinfo(&token.access_token).await {
        Ok(v) => v,
        Err(e) => {
            let redirect_url = format!(
                "{}?google_error=userinfo_failed&google_error_message={}",
                redirect_base,
                urlencoding::encode(&e),
            );

            return HttpResponse::Found()
                .append_header(("Location", redirect_url))
                .finish();
        }
    };

    let calendars = match fetch_google_calendars(&token.access_token).await {
        Ok(v) => v,
        Err(e) => {
            let redirect_url = format!(
                "{}?google_error=calendar_list_failed&google_error_message={}",
                redirect_base,
                urlencoding::encode(&e),
            );

            return HttpResponse::Found()
                .append_header(("Location", redirect_url))
                .finish();
        }
    };

    let session_id = Uuid::new_v4().to_string();

    let session = GoogleOAuthSession {
        session_id: session_id.clone(),
        account_email: userinfo.email,
        provider_account_id: userinfo.sub,
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        expires_in: token.expires_in,
        scopes: token.scope,
        calendars,
        selected_calendar_ids: vec![],
        created_at_unix: Utc::now().timestamp(),
    };

    match state.google_oauth_sessions.lock() {
        Ok(mut sessions) => {
            sessions.insert(session_id.clone(), session);
        }
        Err(_) => return HttpResponse::InternalServerError().body("failed to store google session"),
    }

    let redirect_url = format!(
        "{}?google_session={}",
        redirect_base,
        urlencoding::encode(&session_id)
    );

    HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish()
}

#[get("/google/calendars/session/{session_id}")]
pub async fn google_session_get(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let session_id = path.into_inner();

    let session = match state.google_oauth_sessions.lock() {
        Ok(sessions) => sessions.get(&session_id).cloned(),
        Err(_) => return HttpResponse::InternalServerError().body("failed to read google session"),
    };

    let Some(session) = session else {
        return HttpResponse::NotFound().body("google session not found");
    };

    if !has_google_calendar_scope(session.scopes.as_deref()) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "calendar_scope_denied",
            "message": "Google Calendar permission was not granted."
        }));
    }

    HttpResponse::Ok().json(GoogleSessionView {
        session_id: session.session_id,
        account_email: session.account_email,
        calendars: session.calendars,
        selected_calendar_ids: session.selected_calendar_ids,
    })
}

#[post("/google/calendars/session/{session_id}/select")]
pub async fn google_session_select(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<GoogleSelectRequest>,
) -> impl Responder {
    if body.selected_calendar_ids.is_empty() {
        return HttpResponse::BadRequest().body("select at least one calendar");
    }

    if body.selected_calendar_ids.len() > 2 {
        return HttpResponse::BadRequest().body("you can select at most 2 calendars");
    }

    let session_id = path.into_inner();

    match state.google_oauth_sessions.lock() {
        Ok(mut sessions) => {
            let Some(session) = sessions.get_mut(&session_id) else {
                return HttpResponse::NotFound().body("google session not found");
            };

            if !has_google_calendar_scope(session.scopes.as_deref()) {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "calendar_scope_denied",
                    "message": "Google Calendar permission was not granted."
                }));
            }

            let allowed = session
                .calendars
                .iter()
                .map(|v| v.id.as_str())
                .collect::<std::collections::HashSet<_>>();

            for selected in &body.selected_calendar_ids {
                if !allowed.contains(selected.as_str()) {
                    return HttpResponse::BadRequest().body("invalid calendar selection");
                }
            }

            session.selected_calendar_ids = body.selected_calendar_ids.clone();
        }
        Err(_) => return HttpResponse::InternalServerError().body("failed to update google session"),
    }

    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok"
    }))
}

fn parse_google_state(raw: Option<&String>) -> GoogleOAuthState {
    raw.and_then(|v| serde_json::from_str::<GoogleOAuthState>(v).ok())
        .unwrap_or_default()
}

fn build_google_return_url(state: &GoogleOAuthState) -> String {
    match state.return_to.as_deref() {
        Some("expert_edit") => {
            if let Some(slug) = state.slug.as_deref() {
                format!("/e/{}/edit", urlencoding::encode(slug))
            } else {
                "/expert-new.html".to_string()
            }
        }
        _ => "/expert-new.html".to_string(),
    }
}