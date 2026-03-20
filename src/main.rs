mod db;
mod entities;

use actix_files::Files;
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use db::connect_db;
use hmac::{Hmac, Mac};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
struct AppCfg {
    bot_token: String,
}

#[derive(Clone)]
struct AppState {
    cfg: AppCfg,
    db: DatabaseConnection,
}

#[derive(Deserialize)]
struct TgAuthPayload {
    id: i64,
    first_name: String,
    #[serde(default)]
    last_name: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    photo_url: Option<String>,
    auth_date: i64,
    hash: String,
}

#[post("/tg-auth")]
async fn tg_auth(state: web::Data<AppState>, body: web::Json<TgAuthPayload>) -> HttpResponse {
    let data = body.into_inner();

    let mut pairs: Vec<(&str, String)> = Vec::new();
    pairs.push(("auth_date", data.auth_date.to_string()));
    pairs.push(("first_name", data.first_name.clone()));
    pairs.push(("id", data.id.to_string()));
    if let Some(ref v) = data.last_name {
        pairs.push(("last_name", v.clone()));
    }
    if let Some(ref v) = data.username {
        pairs.push(("username", v.clone()));
    }
    if let Some(ref v) = data.photo_url {
        pairs.push(("photo_url", v.clone()));
    }

    pairs.sort_by(|a, b| a.0.cmp(b.0));

    let data_check_string = pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    let token_trimmed = state.cfg.bot_token.trim();
    let secret_key = Sha256::digest(token_trimmed.as_bytes());

    let mut mac = HmacSha256::new_from_slice(&secret_key).unwrap();
    mac.update(data_check_string.as_bytes());
    let calc_hash = hex::encode(mac.finalize().into_bytes());

    if calc_hash != data.hash.to_lowercase() {
        return HttpResponse::Unauthorized().body("hash mismatch");
    }

    let now = chrono::Utc::now().timestamp();
    if (now - data.auth_date).abs() > 86_400 {
        return HttpResponse::Unauthorized().body("auth expired");
    }

    HttpResponse::Ok().json(serde_json::json!({
        "id": data.id,
        "first_name": data.first_name,
        "last_name": data.last_name,
        "username": data.username,
        "photo_url": data.photo_url
    }))
}

#[derive(Deserialize)]
struct LinkWalletPayload {
    address: String,
    chain: i32,
    #[serde(default)]
    telegram_id: Option<i64>,
}

#[post("/link-wallet")]
async fn link_wallet(body: web::Json<LinkWalletPayload>) -> HttpResponse {
    println!(
        "Link wallet request: tg_id={:?}, addr={}, chain={}",
        body.telegram_id, body.address, body.chain
    );

    HttpResponse::Ok().finish()
}

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "expert-hub",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp_utc": chrono::Utc::now().to_rfc3339(),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = connect_db(&database_url)
        .await
        .expect("failed to connect to database");

    let state = AppState {
        cfg: AppCfg {
            bot_token: std::env::var("TELEGRAM_BOT_TOKEN")
                .expect("TELEGRAM_BOT_TOKEN must be set"),
        },
        db,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(health)
            .service(tg_auth)
            .service(link_wallet)
            .service(Files::new("/", "public").index_file("index.html"))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}