use chrono::{Duration, Utc};

use crate::entities::{
    bookings,
    experts,
    payments,
};
use crate::services::ton::{
    client::TonWorkerClient,
    dto::{
        BookingContractStateResponse,
        ContractActionRequest,
        ContractActionResponse,
        CreateBookingContractRequest,
        CreateBookingContractResponse,
    },
};

use crate::services::booking_rules::{
    calculate_expert_confirmation_deadline,
    calculate_session_outcome_deadline,
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
        amount: String,
        currency: String,
    )-> Result<CreateBookingContractRequest, String> {
        fn map_wallet_for_ton_testnet(wallet: &str) -> String {
            const REAL_CUSTOMER_TEST_WALLET: &str =
                "UQDiZ93j19DQJElNstmAT9mWu72xtFqXfKx_o6OY_JwwjW9l";
            const TESTNET_CUSTOMER_WALLET: &str =
                "0QDiZ93j19DQJElNstmAT9mWu72xtFqXfKx_o6OY_JwwjdTv";

            const REAL_EXPERT_TEST_WALLET: &str =
                "UQB3C7AXeV5AFLFDq-RD-_gZNDfq8oNbhe4PuntHQNAY3xm9";
            const TESTNET_EXPERT_WALLET: &str =
                "0QB3C7AXeV5AFLFDq-RD-_gZNDfq8oNbhe4PuntHQNAY36I3";

            match wallet {
                REAL_CUSTOMER_TEST_WALLET => TESTNET_CUSTOMER_WALLET.to_string(),
                REAL_EXPERT_TEST_WALLET => TESTNET_EXPERT_WALLET.to_string(),
                _ => wallet.to_string(),
            }
        }

         let customer_wallet = booking
                .requested_by_ton_wallet
                .clone()
                .ok_or_else(|| "customer TON wallet is missing on booking".to_string())?;

            let slot_start_unix = booking.slot_start.timestamp();

            let expert_confirmation_deadline_unix =
                calculate_expert_confirmation_deadline(booking.slot_start)?
                    .timestamp();

            let session_outcome_deadline_unix =
                calculate_session_outcome_deadline(booking.slot_end)?
                    .timestamp();

           Ok(CreateBookingContractRequest {
               booking_id: booking.id,
               payment_id: payment.id,
               customer_telegram_id: booking.requested_by_telegram_id,
               expert_telegram_id: expert.telegram_id,
               customer_wallet: map_wallet_for_ton_testnet(&customer_wallet),
               expert_wallet: map_wallet_for_ton_testnet(&expert.ton_wallet_address),
               amount,
               currency,
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

        println!(
            "TON_CONTROLLER_DECLINE contract={} booking={} payment={} reason={:?}",
            contract_address,
            booking_id,
            payment_id,
            reason
        );

        let request = ContractActionRequest {
            payment_id,
            booking_id,
            action: "expert_decline".to_string(),
            reason,
        };

        let result = self.client
            .send_contract_action(
                &contract_address,
                &request,
            )
            .await;

        println!(
            "TON_CONTROLLER_DECLINE_RESULT {:?}",
            result
        );

        result
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

    pub async fn get_booking_contract_state(
        &self,
        contract_address: &str,
    ) -> Result<BookingContractStateResponse, String> {
        self.client.get_booking_contract_state(contract_address).await
    }
}