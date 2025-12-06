use actix_files::Files;
use actix_web::{post, web, App, HttpResponse, HttpServer};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
struct AppCfg {
    bot_token: String, // TELEGRAM_BOT_TOKEN from env
}

/* ---------- Telegram auth payload ---------- */

#[derive(Deserialize)]
struct TgAuthPayload {
    id: i64,
    first_name: String,
    #[serde(default)] last_name: Option<String>,
    #[serde(default)] username: Option<String>,
    #[serde(default)] photo_url: Option<String>,
    auth_date: i64,
    hash: String,
}

/* ---------- /tg-auth: verify login widget data ---------- */

#[post("/tg-auth")]
async fn tg_auth(cfg: web::Data<AppCfg>, body: web::Json<TgAuthPayload>) -> HttpResponse {
    let data = body.into_inner();

    // 1) build data_check_string = sorted key=value lines (exclude hash, skip None)
    let mut pairs: Vec<(&str, String)> = Vec::new();
    pairs.push(("auth_date", data.auth_date.to_string()));
    pairs.push(("first_name", data.first_name.clone()));
    pairs.push(("id", data.id.to_string()));
    if let Some(ref v) = data.last_name { pairs.push(("last_name", v.clone())); }
    if let Some(ref v) = data.username { pairs.push(("username", v.clone())); }
    if let Some(ref v) = data.photo_url { pairs.push(("photo_url", v.clone())); }

    pairs.sort_by(|a, b| a.0.cmp(b.0));

    let data_check_string = pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    // 2) secret_key = SHA256(bot_token)  (Telegram Login Widget docs)
    let token_trimmed = cfg.bot_token.trim(); // strip accidental spaces/newlines
    let secret_key = Sha256::digest(token_trimmed.as_bytes());

    let mut mac = HmacSha256::new_from_slice(&secret_key).unwrap();
    mac.update(data_check_string.as_bytes());
    let calc_hash = hex::encode(mac.finalize().into_bytes());

    if calc_hash != data.hash.to_lowercase() {
        eprintln!(
            "[tg-auth] HASH MISMATCH\n  check_string = {}\n  token = '{}'\n  from_tg = {}\n  calc    = {}",
            data_check_string,
            token_trimmed,
            data.hash,
            calc_hash
        );
        return HttpResponse::Unauthorized().body("hash mismatch");
    }

    // 3) freshness (24h window)
    let now = chrono::Utc::now().timestamp();
    if (now - data.auth_date).abs() > 86_400 {
        eprintln!(
            "[tg-auth] AUTH EXPIRED now={} auth_date={}",
            now, data.auth_date
        );
        return HttpResponse::Unauthorized().body("auth expired");
    }

    // 4) success – return profile for frontend
    HttpResponse::Ok().json(serde_json::json!({
        "id": data.id,
        "first_name": data.first_name,
        "last_name": data.last_name,
        "username": data.username,
        "photo_url": data.photo_url
    }))
}

/* ---------- /link-wallet: TON address mapping ---------- */

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

    // TODO: persist mapping in DB
    HttpResponse::Ok().finish()
}

/* ---------- boot ---------- */

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    let cfg = AppCfg {
        bot_token: std::env::var("TELEGRAM_BOT_TOKEN")
            .expect("TELEGRAM_BOT_TOKEN must be set"),
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cfg.clone()))
            .service(tg_auth)
            .service(link_wallet)
            // Serve everything from ./public (index.html, tonconnect-manifest.json, icon-192.png, terms.txt, privacy.txt, etc.)
            .service(Files::new("/", "public").index_file("index.html"))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}