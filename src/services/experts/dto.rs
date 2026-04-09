use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct UpsertExpertRequest {
    pub telegram_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub photo_url: Option<String>,
    pub display_name: String,
    pub telegram_bio: Option<String>,
    pub ton_wallet_address: String,
    pub timezone: String,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub working_days: Value,
    pub work_start_time: String,
    pub work_end_time: String,
    pub allowed_session_durations: Value,
}

#[derive(Debug)]
pub struct UpsertExpertData {
    pub telegram_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub photo_url: Option<String>,
    pub display_name: String,
    pub telegram_bio: Option<String>,
    pub ton_wallet_address: String,
    pub timezone: String,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub working_days: Value,
    pub work_start_time: String,
    pub work_end_time: String,
    pub allowed_session_durations: Value,
    pub is_bookable: bool,
}

#[derive(Debug, Serialize)]
pub struct UpsertExpertResponse {
    pub id: i64,
    pub telegram_id: i64,
    pub username: String,
    pub display_name: String,
    pub ton_wallet_address: String,
    pub timezone: String,
    pub public_slug: String,
    pub created: bool,
}

#[derive(Debug, Serialize)]
pub struct EditCalendarOption {
    pub id: i64,
    pub provider: String,
    pub connection_label: String,
    pub is_primary: bool,
    pub is_enabled: bool,
    pub selected_calendar_name: Option<String>,
    pub selected_calendar_timezone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EditExpertResponse {
    pub id: i64,
    pub telegram_id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub display_name: String,
    pub telegram_bio: Option<String>,
    pub photo_url: Option<String>,
    pub public_slug: String,
    pub ton_wallet_address: String,
    pub timezone: String,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub working_days: Value,
    pub work_start_time: String,
    pub work_end_time: String,
    pub allowed_session_durations: Value,
    pub minimum_notice_minutes: i32,
    pub buffer_before_minutes: i32,
    pub buffer_after_minutes: i32,
    pub max_days_ahead: i32,
    pub is_active: bool,
    pub is_bookable: bool,
    pub booking_target_strategy: String,
    pub primary_calendar_connection_id: Option<i64>,
    pub calendar_connections: Vec<EditCalendarOption>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExpertProfileRequest {
    pub telegram_id: i64,
    pub display_name: String,
    pub telegram_bio: Option<String>,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub working_days: Value,
    pub work_start_time: String,
    pub work_end_time: String,
    pub allowed_session_durations: Value,
    pub minimum_notice_minutes: i32,
    pub buffer_before_minutes: i32,
    pub buffer_after_minutes: i32,
    pub max_days_ahead: i32,
    pub is_active: bool,
    pub is_bookable: bool,
    pub primary_calendar_connection_id: Option<i64>,
    pub ton_wallet_address: Option<String>,
    #[serde(default)]
    pub attach_google_session_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PublicExpertResponse {
    pub id: i64,
    pub telegram_id: i64,
    pub public_slug: String,
    pub display_name: String,
    pub username: String,
    pub photo_url: Option<String>,
    pub telegram_bio: Option<String>,
    pub timezone: String,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub allowed_session_durations: Value,
    pub working_days: Value,
    pub work_start_time: String,
    pub work_end_time: String,
    pub minimum_notice_minutes: i32,
    pub buffer_before_minutes: i32,
    pub buffer_after_minutes: i32,
    pub max_days_ahead: i32,
    pub is_active: bool,
    pub is_bookable: bool,
    pub expert_rating: Decimal,
    pub reviews_count: i32,
}

#[derive(Debug, Serialize)]
pub struct PopularExpertCardResponse {
    pub public_slug: String,
    pub display_name: String,
    pub username: String,
    pub photo_url: Option<String>,
    pub expert_rating: Decimal,
    pub reviews_count: i32,
}