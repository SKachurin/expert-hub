use actix_web::{post, web, HttpRequest, HttpResponse};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
pub struct TgAuthPayload {
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
pub async fn tg_auth(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<TgAuthPayload>,
) -> HttpResponse {
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

    let host = req.connection_info().host().to_string();

    let bot_token = if host.contains("dev.experthub.bar") {
        state.config.telegram_dev_bot_token.trim()
    } else {
        state.config.telegram_bot_token.trim()
    };

    let secret_key = Sha256::digest(bot_token.as_bytes());

    let mut mac = HmacSha256::new_from_slice(&secret_key).unwrap();
    mac.update(data_check_string.as_bytes());
    let calc_hash = hex::encode(mac.finalize().into_bytes());

    if calc_hash != data.hash.to_lowercase() {
        eprintln!(
            "[tg-auth] HASH MISMATCH\n  host    = {}\n  check_string = {}\n  from_tg = {}\n  calc    = {}",
            host, data_check_string, data.hash, calc_hash
        );
        return HttpResponse::Unauthorized().body("hash mismatch");
    }

    let now = chrono::Utc::now().timestamp();
    if (now - data.auth_date).abs() > 86_400 {
        eprintln!(
            "[tg-auth] AUTH EXPIRED now={} auth_date={}",
            now, data.auth_date
        );
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
pub struct LinkWalletPayload {
    address: String,
    chain: i32,
    #[serde(default)]
    telegram_id: Option<i64>,
}

#[post("/link-wallet")]
pub async fn link_wallet(body: web::Json<LinkWalletPayload>) -> HttpResponse {
    println!(
        "Link wallet request: tg_id={:?}, addr={}, chain={}",
        body.telegram_id, body.address, body.chain
    );

    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok"
    }))
}