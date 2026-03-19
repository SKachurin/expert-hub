#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub telegram_bot_token: String,
    pub app_env: String,
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

        let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

        Self {
            host,
            port,
            telegram_bot_token,
            app_env,
        }
    }
}