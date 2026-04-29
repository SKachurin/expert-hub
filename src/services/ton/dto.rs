use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBookingContractRequest {
    pub booking_id: i64,
    pub payment_id: i64,
    pub customer_telegram_id: i64,
    pub expert_telegram_id: i64,
    pub customer_wallet: String,
    pub expert_wallet: String,

    pub amount: String,
    pub currency: String,

    pub slot_start_unix: i64,
    pub expert_confirmation_deadline_unix: i64,
    pub session_outcome_deadline_unix: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBookingContractResponse {
    pub contract_address: String,
    pub state_init_boc: String,
    pub amount_nano_ton: String,
    pub recommended_gas_buffer_nano_ton: String,
    pub total_deploy_value_nano_ton: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractActionRequest {
    pub payment_id: i64,
    pub booking_id: i64,
    pub action: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractActionResponse {
    pub contract_address: String,
    pub action: String,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingContractStateResponse {
    pub contract_address: String,
    pub account_state: String,
    pub balance_nano_ton: String,
    pub contract_state: Option<i32>,
    pub is_funded: bool,
}