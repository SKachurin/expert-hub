use chrono::{DateTime, Duration, FixedOffset, Utc};

pub const PLATFORM_MIN_BOOKING_LEAD_MINUTES: i64 = 4 * 60;
pub const LONG_SLOT_THRESHOLD_MINUTES: i64 = 24 * 60;
pub const LONG_EXPERT_CONFIRM_MINUTES: i64 = 24 * 60;
pub const SHORT_EXPERT_CONFIRM_MINUTES: i64 = 4 * 60;
pub const LATEST_CONFIRM_BEFORE_SLOT_MINUTES: i64 = 30;
pub const SESSION_OUTCOME_GRACE_MINUTES: i64 = 10;

pub const REJECTED_REASON_EXPERT_DECLINED: &str = "expert_declined";
pub const REJECTED_REASON_EXPERT_RESPONSE_TIMEOUT: &str = "expert_response_timeout";

pub fn ensure_slot_has_platform_min_lead(
    slot_start: DateTime<FixedOffset>,
) -> Result<(), String> {
    let now = Utc::now();
    let slot_start_utc = slot_start.with_timezone(&Utc);

    if slot_start_utc < now + Duration::minutes(PLATFORM_MIN_BOOKING_LEAD_MINUTES) {
        return Err("selected slot must start at least 4 hours from now".to_string());
    }

    Ok(())
}

pub fn calculate_expert_confirmation_deadline(
    slot_start: DateTime<FixedOffset>,
) -> Result<DateTime<Utc>, String> {
    let now = Utc::now();
    let slot_start_utc = slot_start.with_timezone(&Utc);

    let minutes_until_slot = (slot_start_utc - now).num_minutes();

    if minutes_until_slot < PLATFORM_MIN_BOOKING_LEAD_MINUTES {
        return Err("slot must start at least 4 hours from now".to_string());
    }

    let response_window = if minutes_until_slot >= LONG_SLOT_THRESHOLD_MINUTES {
        Duration::minutes(LONG_EXPERT_CONFIRM_MINUTES)
    } else {
        Duration::minutes(SHORT_EXPERT_CONFIRM_MINUTES)
    };

    let raw_deadline = now
        .checked_add_signed(response_window)
        .ok_or_else(|| "failed to calculate expert confirmation deadline".to_string())?;

    let latest_safe_deadline = slot_start_utc
        .checked_sub_signed(Duration::minutes(LATEST_CONFIRM_BEFORE_SLOT_MINUTES))
        .ok_or_else(|| "failed to calculate latest safe confirmation deadline".to_string())?;

    Ok(raw_deadline.min(latest_safe_deadline))
}

pub fn calculate_session_outcome_deadline(
    slot_end: DateTime<FixedOffset>,
) -> Result<DateTime<FixedOffset>, String> {
    slot_end
        .checked_add_signed(Duration::minutes(SESSION_OUTCOME_GRACE_MINUTES))
        .ok_or_else(|| "failed to calculate session outcome deadline".to_string())
}