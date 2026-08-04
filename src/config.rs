use std::env;
use rust_decimal::Decimal;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,

    pub telegram_bot_username: String,
    pub telegram_bot_token: String,
    pub telegram_dev_bot_token: String,
    pub active_telegram_bot_token: String,

    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
    pub ton_worker_base_url: String,
    pub ton_worker_auth_token: String,
    pub telegram_webhook_secret_token: String,

    pub platform_fee_percent: Decimal,
    pub ton_gas_reserve: Decimal,
    pub ton_controller_action_reserve: Decimal,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8080);

        let telegram_bot_username = env::var("TELEGRAM_BOT_USERNAME")
            .unwrap_or_default()
            .trim()
            .trim_start_matches('@')
            .to_string();

        let telegram_bot_token = env::var("TELEGRAM_BOT_TOKEN")
            .unwrap_or_default()
            .trim()
            .to_string();

        let telegram_dev_bot_token = env::var("TELEGRAM_DEV_BOT_TOKEN")
            .unwrap_or_default()
            .trim()
            .to_string();

        let active_telegram_bot_token = if telegram_bot_username.is_empty()
            || telegram_bot_username == "expert_hub_bot"
        {
            telegram_dev_bot_token.clone()
        } else {
            telegram_bot_token.clone()
        };

        if active_telegram_bot_token.is_empty() {
            panic!(
                "active Telegram bot token is empty. TELEGRAM_BOT_USERNAME=[{}], TELEGRAM_BOT_TOKEN length={}, TELEGRAM_DEV_BOT_TOKEN length={}",
                telegram_bot_username,
                telegram_bot_token.len(),
                telegram_dev_bot_token.len()
            );
        }

        let google_client_id = env::var("GOOGLE_CLIENT_ID")
            .expect("GOOGLE_CLIENT_ID must be set");

        let google_client_secret = env::var("GOOGLE_CLIENT_SECRET")
            .expect("GOOGLE_CLIENT_SECRET must be set");

        let google_redirect_uri = env::var("GOOGLE_REDIRECT_URI")
            .expect("GOOGLE_REDIRECT_URI must be set");

        let ton_worker_base_url = env::var("TON_WORKER_BASE_URL")
            .unwrap_or_else(|_| "http://ton-worker:8081".to_string());

        let ton_worker_auth_token = env::var("TON_WORKER_AUTH_TOKEN")
            .unwrap_or_default()
            .trim()
            .to_string();

        let telegram_webhook_secret_token = env::var("TELEGRAM_WEBHOOK_SECRET_TOKEN")
            .unwrap_or_default()
            .trim()
            .to_string();

        let platform_fee_percent = env::var("PLATFORM_FEE_PERCENT")
            .unwrap_or_else(|_| "1.0".to_string())
            .trim()
            .parse::<Decimal>()
            .expect("PLATFORM_FEE_PERCENT must be a valid decimal number");

        let ton_gas_reserve = env::var("TON_GAS_RESERVE")
            .unwrap_or_else(|_| "0.02".to_string())
            .trim()
            .parse::<Decimal>()
            .expect("TON_GAS_RESERVE must be a valid decimal number");

        let ton_controller_action_reserve = env::var("TON_CONTROLLER_ACTION_RESERVE")
            .unwrap_or_else(|_| "0.05".to_string())
            .trim()
            .parse::<Decimal>()
            .expect("TON_CONTROLLER_ACTION_RESERVE must be a valid decimal number");

        Self {
            host,
            port,
            telegram_bot_username,
            telegram_bot_token,
            telegram_dev_bot_token,
            active_telegram_bot_token,
            google_client_id,
            google_client_secret,
            google_redirect_uri,
            ton_worker_base_url,
            ton_worker_auth_token,
            telegram_webhook_secret_token,
            platform_fee_percent,
            ton_gas_reserve,
            ton_controller_action_reserve,
        }
    }
}