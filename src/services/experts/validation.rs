use super::dto::{UpdateExpertProfileRequest, UpsertExpertData};

pub fn validate_update_data(data: &UpdateExpertProfileRequest) -> Result<(), String> {
    if data.telegram_id <= 0 {
        return Err("telegram_id is required".to_string());
    }

    if data.display_name.trim().is_empty() {
        return Err("display_name is required".to_string());
    }

    if data.currency.trim().is_empty() {
        return Err("currency is required".to_string());
    }

    if !data.working_days.is_array() {
        return Err("working_days must be a JSON array".to_string());
    }

    if !data.allowed_session_durations.is_array() {
        return Err("allowed_session_durations must be a JSON array".to_string());
    }

    if data.minimum_notice_minutes < 0
        || data.buffer_before_minutes < 0
        || data.buffer_after_minutes < 0
        || data.max_days_ahead < 1
    {
        return Err("invalid scheduling numbers".to_string());
    }

    Ok(())
}

pub fn validate_data(data: &UpsertExpertData) -> Result<(), String> {
    if data.telegram_id <= 0 {
        return Err("telegram_id is required".to_string());
    }

    if data.username.trim().is_empty() {
        return Err("username is required".to_string());
    }

    if data.ton_wallet_address.trim().is_empty() {
        return Err("ton_wallet_address is required".to_string());
    }

    if data.timezone.trim().is_empty() {
        return Err("timezone is required".to_string());
    }

    if !data.working_days.is_array() {
        return Err("working_days must be a JSON array".to_string());
    }

    if !data.allowed_session_durations.is_array() {
        return Err("allowed_session_durations must be a JSON array".to_string());
    }

    Ok(())
}