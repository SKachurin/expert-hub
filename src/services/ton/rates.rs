use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct CoinGeckoSimplePriceResponse {
    #[serde(rename = "the-open-network")]
    ton: CoinGeckoTonPrice,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoTonPrice {
    usd: Decimal,
}

pub async fn fetch_ton_usd_rate() -> Result<Decimal, String> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://api.coingecko.com/api/v3/simple/price")
        .query(&[
            ("ids", "the-open-network"),
            ("vs_currencies", "usd"),
        ])
        .header(
            reqwest::header::USER_AGENT,
            "ExpertHub/0.1 payment-rate-service contact: support@experthub.bar",
        )
        .send()
        .await
        .map_err(|e| format!("TON/USD rate request failed: {e}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("failed to read TON/USD rate response: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "TON/USD rate request failed with {status}: {text}"
        ));
    }

    let parsed: CoinGeckoSimplePriceResponse = serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse TON/USD rate response: {e}. Body: {text}"))?;

    if parsed.ton.usd <= Decimal::ZERO {
        return Err("TON/USD rate must be positive".to_string());
    }

    Ok(parsed.ton.usd)
}

pub fn convert_usd_to_ton_amount(
    usd_amount: Decimal,
    ton_usd_rate: Decimal,
) -> Result<Decimal, String> {
    if usd_amount <= Decimal::ZERO {
        return Err("USD amount must be positive".to_string());
    }

    if ton_usd_rate <= Decimal::ZERO {
        return Err("TON/USD rate must be positive".to_string());
    }

    Ok(usd_amount / ton_usd_rate)
}