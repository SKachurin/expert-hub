use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use reqwest::{Client, StatusCode};

use crate::config::AppConfig;
use crate::state::GoogleCalendarCandidate;

const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_FREEBUSY_URL: &str = "https://www.googleapis.com/calendar/v3/freeBusy";
const GOOGLE_OAUTH_AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_CALENDAR_LIST_URL: &str = "https://www.googleapis.com/calendar/v3/users/me/calendarList";
const GOOGLE_USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";
const GOOGLE_SCOPE: &str = "openid email https://www.googleapis.com/auth/calendar.readonly";

#[derive(Debug, Deserialize)]
pub struct GoogleRefreshTokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub expires_in: Option<i64>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GoogleBusyInterval {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct GoogleFreeBusyRequest<'a> {
    #[serde(rename = "timeMin")]
    time_min: String,
    #[serde(rename = "timeMax")]
    time_max: String,
    #[serde(rename = "timeZone")]
    time_zone: &'a str,
    items: Vec<GoogleFreeBusyItem<'a>>,
}

#[derive(Debug, Serialize)]
struct GoogleFreeBusyItem<'a> {
    id: &'a str,
}

#[derive(Debug, Deserialize)]
struct GoogleFreeBusyResponse {
    calendars: HashMap<String, GoogleFreeBusyCalendar>,
}

#[derive(Debug, Deserialize)]
struct GoogleFreeBusyCalendar {
    #[serde(default)]
    busy: Vec<GoogleFreeBusyBusyItem>,
}

#[derive(Debug, Deserialize)]
struct GoogleFreeBusyBusyItem {
    start: String,
    end: String,
}


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

pub async fn fetch_google_free_busy(
    access_token: &str,
    calendar_ids: &[String],
    time_min: DateTime<Utc>,
    time_max: DateTime<Utc>,
    time_zone: &str,
) -> Result<Vec<GoogleBusyInterval>, String> {
    if calendar_ids.is_empty() {
        return Ok(Vec::new());
    }

    let client = Client::new();

    let body = GoogleFreeBusyRequest {
        time_min: time_min.to_rfc3339(),
        time_max: time_max.to_rfc3339(),
        time_zone,
        items: calendar_ids
            .iter()
            .map(|id| GoogleFreeBusyItem { id: id.as_str() })
            .collect(),
    };

    let resp = client
        .post(GOOGLE_FREEBUSY_URL)
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("google freeBusy request failed: {e}"))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("google freeBusy failed: {}", text));
    }

    let parsed = resp
        .json::<GoogleFreeBusyResponse>()
        .await
        .map_err(|e| format!("failed to parse google freeBusy response: {e}"))?;

    let mut intervals = Vec::new();

    for (_, calendar) in parsed.calendars {
        for busy in calendar.busy {
            let start = DateTime::parse_from_rfc3339(&busy.start)
                .map_err(|e| format!("invalid google busy start: {e}"))?
                .with_timezone(&Utc);

            let end = DateTime::parse_from_rfc3339(&busy.end)
                .map_err(|e| format!("invalid google busy end: {e}"))?
                .with_timezone(&Utc);

            if end > start {
                intervals.push(GoogleBusyInterval { start, end });
            }
        }
    }

    Ok(intervals)
}

pub async fn refresh_google_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<(String, Option<DateTime<Utc>>), String> {
    let client = Client::new();

    let response = client
        .post(GOOGLE_TOKEN_URL)
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| format!("google token refresh request failed: {e}"))?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("google token refresh failed: {text}"));
    }

    let parsed = response
        .json::<GoogleRefreshTokenResponse>()
        .await
        .map_err(|e| format!("failed to parse google refresh response: {e}"))?;

    let expires_at = parsed
        .expires_in
        .map(|seconds| Utc::now() + Duration::seconds(seconds.saturating_sub(60)));

    Ok((parsed.access_token, expires_at))
}

pub async fn fetch_google_free_busy_raw(
    access_token: &str,
    calendar_ids: &[String],
    time_min: DateTime<Utc>,
    time_max: DateTime<Utc>,
    time_zone: &str,
) -> Result<reqwest::Response, String> {
    if calendar_ids.is_empty() {
        return Err("calendar_ids is empty".to_string());
    }

    let client = Client::new();

    let body = GoogleFreeBusyRequest {
        time_min: time_min.to_rfc3339(),
        time_max: time_max.to_rfc3339(),
        time_zone,
        items: calendar_ids
            .iter()
            .map(|id| GoogleFreeBusyItem { id: id.as_str() })
            .collect(),
    };

    client
        .post(GOOGLE_FREEBUSY_URL)
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("google freeBusy request failed: {e}"))
}

pub async fn parse_google_free_busy_response(
    resp: reqwest::Response,
) -> Result<Vec<GoogleBusyInterval>, String> {
    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("google freeBusy failed: {text}"));
    }

    let parsed = resp
        .json::<GoogleFreeBusyResponse>()
        .await
        .map_err(|e| format!("failed to parse google freeBusy response: {e}"))?;

    let mut intervals = Vec::new();

    for (_, calendar) in parsed.calendars {
        for busy in calendar.busy {
            let start = DateTime::parse_from_rfc3339(&busy.start)
                .map_err(|e| format!("invalid google busy start: {e}"))?
                .with_timezone(&Utc);

            let end = DateTime::parse_from_rfc3339(&busy.end)
                .map_err(|e| format!("invalid google busy end: {e}"))?
                .with_timezone(&Utc);

            if end > start {
                intervals.push(GoogleBusyInterval { start, end });
            }
        }
    }

    Ok(intervals)
}