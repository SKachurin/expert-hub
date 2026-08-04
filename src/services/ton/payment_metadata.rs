use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use serde_json::Value;

use crate::entities::bookings;

#[derive(Debug, Clone)]
pub struct TonPaymentMetadata {
    pub contract_address: String,
    pub expert_amount_nano_ton: u128,
    pub platform_fee_nano_ton: u128,
    pub gas_reserve_nano_ton: u128,
    pub controller_reserve_nano_ton: u128,
    pub customer_total_nano_ton: u128,
    pub wallet_send_amount_nano_ton: String,
    pub state_init_boc: String,
    pub recommended_gas_buffer_nano_ton: String,
    pub total_deploy_value_nano_ton: String,
    pub expert_confirmation_deadline: DateTime<FixedOffset>,
    pub session_outcome_deadline: DateTime<FixedOffset>,
}

impl TonPaymentMetadata {
    pub fn from_booking(
        booking: &bookings::Model,
    ) -> Result<Self, String> {
        let metadata = booking
            .metadata
            .as_ref()
            .ok_or_else(|| "booking metadata is missing".to_string())?;

        Self::from_metadata(metadata)
    }

    pub fn from_metadata(
        metadata: &Value,
    ) -> Result<Self, String> {
        let ton = metadata
            .get("ton_payment")
            .ok_or_else(|| "booking metadata is missing ton_payment".to_string())?;

        let payment = Self {
            contract_address: get_string(
                ton,
                "contract_address",
            )?,

            expert_amount_nano_ton: get_u128(
                ton,
                "expert_amount_nano_ton",
            )?,

            platform_fee_nano_ton: get_u128(
                ton,
                "platform_fee_nano_ton",
            )?,

            gas_reserve_nano_ton: get_u128(
                ton,
                "gas_reserve_nano_ton",
            )?,

            controller_reserve_nano_ton: get_u128(
                ton,
                "controller_reserve_nano_ton",
            )?,

            customer_total_nano_ton: get_u128(
                ton,
                "customer_total_nano_ton",
            )?,

            wallet_send_amount_nano_ton: get_string(
                ton,
                "wallet_send_amount_nano_ton",
            )?,

            state_init_boc: get_string(
                ton,
                "state_init_boc",
            )?,

            recommended_gas_buffer_nano_ton: get_string(
                ton,
                "recommended_gas_buffer_nano_ton",
            )?,

            total_deploy_value_nano_ton: get_string(
                ton,
                "total_deploy_value_nano_ton",
            )?,

            expert_confirmation_deadline:
                get_timestamp(
                    ton,
                    "expert_confirmation_deadline_unix",
                )?,

            session_outcome_deadline:
                get_timestamp(
                    ton,
                    "session_outcome_deadline_unix",
                )?,
        };
        payment.validate()?;

        Ok(payment)
    }

    pub fn validate(&self) -> Result<(), String> {
        let calculated = self
            .expert_amount_nano_ton
            .checked_add(self.platform_fee_nano_ton)
            .and_then(|v| v.checked_add(self.gas_reserve_nano_ton))
            .and_then(|v| v.checked_add(self.controller_reserve_nano_ton))
            .ok_or_else(|| {
                "nanoTON overflow".to_string()
            })?;

        if calculated != self.customer_total_nano_ton {
            return Err(
                "customer_total_nano_ton does not equal expert + platform + gas + controller"
                    .to_string(),
            );
        }

        Ok(())
    }
}

fn get_string(
    value: &Value,
    field: &str,
) -> Result<String, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("missing {}", field))
}

fn get_u128(
    value: &Value,
    field: &str,
) -> Result<u128, String> {
    let string = get_string(value, field)?;

    string
        .parse::<u128>()
        .map_err(|e| format!("invalid {}: {}", field, e))
}

fn get_timestamp(
    value: &Value,
    field: &str,
) -> Result<DateTime<FixedOffset>, String> {
    let unix = value
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("missing {}", field))?;

    let utc = Utc
        .timestamp_opt(unix, 0)
        .single()
        .ok_or_else(|| format!("invalid {}", field))?;

    Ok(utc.fixed_offset())
}