use reqwest::Client;
use serde::Deserialize;

use crate::config::AppConfig;
use crate::state::GoogleCalendarCandidate;

const GOOGLE_OAUTH_AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_CALENDAR_LIST_URL: &str = "https://www.googleapis.com/calendar/v3/users/me/calendarList";
const GOOGLE_USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";
const GOOGLE_SCOPE: &str = "openid email https://www.googleapis.com/auth/calendar.readonly";

pub fn build_google_oauth_url(config: &AppConfig, state: &str) -> String {
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&include_granted_scopes=true&prompt=consent&state={}",
        GOOGLE_OAUTH_AUTHORIZE_URL,
        urlencoding::encode(&config.google_client_id),
        urlencoding::encode(&config.google_redirect_uri),
        urlencoding::encode(GOOGLE_SCOPE),
        urlencoding::encode(state),
    )
}

pub fn has_google_calendar_scope(scope_value: Option<&str>) -> bool {
    let Some(scope_value) = scope_value else {
        return false;
    };

    scope_value.split_whitespace().any(|scope| {
        scope == "https://www.googleapis.com/auth/calendar"
            || scope == "https://www.googleapis.com/auth/calendar.readonly"
    })
}

#[derive(Debug, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<i64>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    #[serde(default)]
    pub sub: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleCalendarListResponse {
    #[serde(default)]
    items: Vec<GoogleCalendarItem>,
}

#[derive(Debug, Deserialize)]
struct GoogleCalendarItem {
    id: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    #[allow(non_snake_case)]
    timeZone: Option<String>,
    #[serde(default)]
    #[allow(non_snake_case)]
    accessRole: Option<String>,
    #[serde(default)]
    primary: Option<bool>,
}

pub async fn exchange_code_for_tokens(
    config: &AppConfig,
    code: &str,
) -> Result<GoogleTokenResponse, String> {
    let client = Client::new();

    let resp = client
        .post(GOOGLE_OAUTH_TOKEN_URL)
        .form(&[
            ("client_id", config.google_client_id.as_str()),
            ("client_secret", config.google_client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", config.google_redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("google token request failed: {e}"))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("google token exchange failed: {}", text));
    }

    resp.json::<GoogleTokenResponse>()
        .await
        .map_err(|e| format!("failed to parse google token response: {e}"))
}

pub async fn fetch_google_userinfo(access_token: &str) -> Result<GoogleUserInfo, String> {
    let client = Client::new();

    let resp = client
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("google userinfo request failed: {e}"))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("google userinfo failed: {}", text));
    }

    resp.json::<GoogleUserInfo>()
        .await
        .map_err(|e| format!("failed to parse google userinfo: {e}"))
}

pub async fn fetch_google_calendars(access_token: &str) -> Result<Vec<GoogleCalendarCandidate>, String> {
    let client = Client::new();

    let resp = client
        .get(GOOGLE_CALENDAR_LIST_URL)
        .query(&[("minAccessRole", "reader")])
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("google calendar list request failed: {e}"))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("google calendar list failed: {}", text));
    }

    let parsed = resp
        .json::<GoogleCalendarListResponse>()
        .await
        .map_err(|e| format!("failed to parse google calendar list: {e}"))?;

    Ok(parsed
        .items
        .into_iter()
        .map(|item| GoogleCalendarCandidate {
            id: item.id,
            summary: item.summary.unwrap_or_else(|| "Unnamed calendar".to_string()),
            time_zone: item.timeZone,
            access_role: item.accessRole,
            primary: item.primary.unwrap_or(false),
        })
        .collect())
}