use chrono::{Duration, Utc};

use crate::entities::{
    bookings,
    experts,
    payments,
};
use crate::services::ton::{
    client::TonWorkerClient,
    dto::{
        ContractActionRequest,
        ContractActionResponse,
        CreateBookingContractRequest,
        CreateBookingContractResponse,
    },
};

pub struct TonController {
    pub client: TonWorkerClient,
}

impl TonController {
    pub fn new(client: TonWorkerClient) -> Self {
        Self { client }
    }

    pub fn build_create_booking_contract_request(
        booking: &bookings::Model,
        payment: &payments::Model,
        expert: &experts::Model,
        amount_nano_ton: String,
    ) -> Result<CreateBookingContractRequest, String> {
        let customer_wallet = booking
            .requested_by_ton_wallet
            .clone()
            .ok_or_else(|| "customer TON wallet is missing on booking".to_string())?;

        let slot_start_unix = booking.slot_start.timestamp();

        let expert_confirmation_deadline_unix = Utc::now()
            .checked_add_signed(Duration::minutes(15))
            .ok_or_else(|| "failed to calculate expert confirmation deadline".to_string())?
            .timestamp();

        let session_outcome_deadline_unix = booking
            .slot_end
            .checked_add_signed(Duration::minutes(10))
            .ok_or_else(|| "failed to calculate session outcome deadline".to_string())?
            .timestamp();

        Ok(CreateBookingContractRequest {
            booking_id: booking.id,
            payment_id: payment.id,
            customer_telegram_id: booking.requested_by_telegram_id,
            expert_telegram_id: expert.telegram_id,
            customer_wallet,
            expert_wallet: expert.ton_wallet_address.clone(),
            amount_nano_ton,
            slot_start_unix,
            expert_confirmation_deadline_unix,
            session_outcome_deadline_unix,
        })
    }

    pub async fn create_booking_contract(
        &self,
        payload: CreateBookingContractRequest,
    ) -> Result<CreateBookingContractResponse, String> {
        self.client.create_booking_contract(&payload).await
    }

    pub async fn confirm_expert(
        &self,
        contract_address: String,
        payment_id: i64,
        booking_id: i64,
    ) -> Result<ContractActionResponse, String> {
        self.client
            .send_contract_action(
                &contract_address,
                &ContractActionRequest {
                    payment_id,
                    booking_id,
                    action: "expert_confirm".to_string(),
                    reason: None,
                },
            )
            .await
    }

    pub async fn decline_expert(
        &self,
        contract_address: String,
        payment_id: i64,
        booking_id: i64,
        reason: Option<String>,
    ) -> Result<ContractActionResponse, String> {
        self.client
            .send_contract_action(
                &contract_address,
                &ContractActionRequest {
                    payment_id,
                    booking_id,
                    action: "expert_decline".to_string(),
                    reason,
                },
            )
            .await
    }

    pub async fn settle_customer_no_show(
        &self,
        contract_address: String,
        payment_id: i64,
        booking_id: i64,
    ) -> Result<ContractActionResponse, String> {
        self.client
            .send_contract_action(
                &contract_address,
                &ContractActionRequest {
                    payment_id,
                    booking_id,
                    action: "customer_no_show".to_string(),
                    reason: None,
                },
            )
            .await
    }

    pub async fn settle_expert_no_show(
        &self,
        contract_address: String,
        payment_id: i64,
        booking_id: i64,
    ) -> Result<ContractActionResponse, String> {
        self.client
            .send_contract_action(
                &contract_address,
                &ContractActionRequest {
                    payment_id,
                    booking_id,
                    action: "expert_no_show".to_string(),
                    reason: None,
                },
            )
            .await
    }

    pub async fn settle_session_connected(
        &self,
        contract_address: String,
        payment_id: i64,
        booking_id: i64,
    ) -> Result<ContractActionResponse, String> {
        self.client
            .send_contract_action(
                &contract_address,
                &ContractActionRequest {
                    payment_id,
                    booking_id,
                    action: "session_connected".to_string(),
                    reason: None,
                },
            )
            .await
    }
}