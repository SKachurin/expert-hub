use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub telegram_bot_token: String,
    pub telegram_dev_bot_token: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
    pub ton_worker_base_url: String,
    pub ton_worker_auth_token: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = std::env::var("PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8080);

        let telegram_bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .expect("TELEGRAM_BOT_TOKEN must be set");

        let telegram_dev_bot_token = std::env::var("TELEGRAM_DEV_BOT_TOKEN")
            .expect("TELEGRAM_DEV_BOT_TOKEN must be set");

        let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
            .expect("GOOGLE_CLIENT_ID must be set");

        let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .expect("GOOGLE_CLIENT_SECRET must be set");

        let google_redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
            .expect("GOOGLE_REDIRECT_URI must be set");

        let ton_worker_base_url = env::var("TON_WORKER_BASE_URL")
            .unwrap_or_else(|_| "http://ton-worker:8081".to_string());

        let ton_worker_auth_token = env::var("TON_WORKER_AUTH_TOKEN")
            .unwrap_or_default();

        Self {
            host,
            port,
            telegram_bot_token,
            telegram_dev_bot_token,
            google_client_id,
            google_client_secret,
            google_redirect_uri,
            ton_worker_base_url,
            ton_worker_auth_token,
        }
    }
}