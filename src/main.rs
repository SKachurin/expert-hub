use actix_files::NamedFile;
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone)]
struct AppCfg {
    bot_token: String, // only the token is needed server-side
}

/* ---------- static page ---------- */
#[get("/")]
async fn index() -> actix_web::Result<NamedFile> {
    // put your index.html at ./public/index.html
    Ok(NamedFile::open("public/index.html")?)
}

/* ---------- OAuth verify ---------- */
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

#[post("/tg-auth")]
async fn tg_auth(cfg: web::Data<AppCfg>, body: web::Json<TgAuthPayload>) -> HttpResponse {
    // build data_check_string (sorted, exclude hash)
    let mut kv = BTreeMap::<&str, String>::new();
    kv.insert("auth_date", body.auth_date.to_string());
    kv.insert("first_name", body.first_name.clone());
    kv.insert("id", body.id.to_string());
    if let Some(v) = &body.last_name { kv.insert("last_name", v.clone()); }
    if let Some(v) = &body.username { kv.insert("username", v.clone()); }
    if let Some(v) = &body.photo_url { kv.insert("photo_url", v.clone()); }

    let data_check_string = kv.iter()
        .map(|(k,v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    // secret key = SHA256(bot_token); HMAC-SHA256 over data_check_string
    let secret = Sha256::digest(cfg.bot_token.as_bytes());
    let mut mac = Hmac::<Sha256>::new_from_slice(&secret).unwrap();
    mac.update(data_check_string.as_bytes());
    let calc = hex::encode(mac.finalize().into_bytes());

    if calc != body.hash.to_lowercase() {
        return HttpResponse::Unauthorized().finish();
    }

    // optional freshness check (24h)
    let now = chrono::Utc::now().timestamp();
    if (now - body.auth_date).unsigned_abs() > 86_400 {
        return HttpResponse::Unauthorized().body("auth expired");
    }

    HttpResponse::Ok().json(serde_json::json!({
        "first_name": body.first_name,
        "last_name": body.last_name,
        "username": body.username,
        "photo_url": body.photo_url
    }))
}

/* ---------- boot ---------- */
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
    let cfg = AppCfg {
        bot_token: std::env::var("TELEGRAM_BOT_TOKEN")
            .expect("TELEGRAM_BOT_TOKEN must be set"),
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cfg.clone()))
            .service(index)
            .service(tg_auth)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
