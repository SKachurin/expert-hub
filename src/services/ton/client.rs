use reqwest::Client;

use crate::services::ton::dto::{
    ContractActionRequest,
    ContractActionResponse,
    CreateBookingContractRequest,
    CreateBookingContractResponse,
};

#[derive(Clone)]
pub struct TonWorkerClient {
    pub base_url: String,
    pub auth_token: String,
    pub http: Client,
}

impl TonWorkerClient {
    pub fn new(base_url: String, auth_token: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token,
            http: Client::new(),
        }
    }

    pub async fn create_booking_contract(
        &self,
        payload: &CreateBookingContractRequest,
    ) -> Result<CreateBookingContractResponse, String> {
        let url = format!("{}/contracts/prepare-booking", self.base_url);

        let response = self.http
            .post(url)
            .header("x-ton-worker-token", &self.auth_token)
            .json(payload)
            .send()
            .await
            .map_err(|e| format!("ton worker prepare request failed: {e}"))?;

        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("ton worker prepare failed with {status}: {text}"));
        }

        response
            .json::<CreateBookingContractResponse>()
            .await
            .map_err(|e| format!("invalid ton worker prepare response: {e}"))
    }

    pub async fn send_contract_action(
        &self,
        contract_address: &str,
        payload: &ContractActionRequest,
    ) -> Result<ContractActionResponse, String> {
        let url = format!("{}/contracts/{}/action", self.base_url, contract_address);

        let response = self.http
            .post(url)
            .header("x-ton-worker-token", &self.auth_token)
            .json(payload)
            .send()
            .await
            .map_err(|e| format!("ton worker action request failed: {e}"))?;

        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("ton worker action failed with {status}: {text}"));
        }

        response
            .json::<ContractActionResponse>()
            .await
            .map_err(|e| format!("invalid ton worker action response: {e}"))
    }
}