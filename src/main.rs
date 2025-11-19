use actix_files::NamedFile;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone)]
struct AppCfg {
    bot_username: String,
    bot_token: String,
}

#[get("/")]
async fn index() -> impl Responder {
    // Single, black page. Telegram login widget injects the button.
    let html = r#"
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>Expert Hub</title>
<style>
  :root { color-scheme: dark; }
  html, body { margin:0; height:100%; background:#000; color:#fff; }
  .wrap { min-height:100%; display:flex; align-items:center; justify-content:center; padding:24px; }
  .card { width:100%; max-width:520px; background:#111; border:1px solid #222; border-radius:16px; padding:24px; box-shadow:0 6px 20px rgba(0,0,0,.5); }
  h1 { font: 600 20px/1.3 system-ui, -apple-system, Segoe UI, Roboto, sans-serif; margin:0 0 12px; }
  p { margin:0 0 16px; color:#bbb; }
  .user { display:flex; gap:14px; align-items:center; margin-top:14px; }
  .avatar { width:56px; height:56px; border-radius:50%; background:#333; display:grid; place-items:center; font-weight:700; }
  .name { font-size:18px; }
  .btnrow { margin-top:16px; }
  .notice { font-size:12px; color:#777; margin-top:10px; }
</style>
</head>
<body>
<div class="wrap">
  <div class="card">
    <h1>Sign in with Telegram</h1>
    <p>We only need your public profile (name & photo).</p>

    <div id="tg-button" class="btnrow">
      <!-- Telegram will place the button here -->
    </div>

    <div id="result" class="user" style="display:none">
      <img id="avatar" class="avatar" alt="">
      <div>
        <div id="fullname" class="name"></div>
        <div id="username" style="color:#888"></div>
      </div>
    </div>

    <div class="notice">If the button doesn’t appear, open this page in a normal browser (or Telegram in-app browser) and make sure your bot’s domain is set in BotFather.</div>
  </div>
</div>

<script>
  const BOT = 'expert_hub_bot'; // no @
  const origin = location.origin; // https://uncategorical-interradially-dawson.ngrok-free.dev
  const iframe = document.createElement('iframe');
  iframe.src =
    `https://oauth.telegram.org/embed/${BOT}?origin=${encodeURIComponent(origin)}&request_access=write&embed=1`;
  iframe.width = 220;
  iframe.height = 54;
  iframe.style.border = '0';
  iframe.style.overflow = 'hidden';
  iframe.setAttribute('scrolling', 'no');
  iframe.setAttribute('allow', 'clipboard-read; clipboard-write');

  document.getElementById('tg-login').appendChild(iframe);

  // receive auth result
  window.addEventListener('message', (e) => {
    if (e.origin !== 'https://oauth.telegram.org') return;
    const data = e.data;
    if (data && data.event === 'auth_result' && data.result) {
      // data.result contains id, first_name, username, photo_url, auth_date, hash
      fetch('/tg-auth', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data.result)
      }).then(() => renderUser(data.result));
    }
  });

  function renderUser(u) {
    document.body.innerHTML =
      `<div style="color:#fff;padding:24px;font:16px/1.4 system-ui">
         <img src="${u.photo_url || ''}" style="width:64px;height:64px;border-radius:50%;vertical-align:middle;background:#222">
         <span style="margin-left:12px;vertical-align:middle">${u.first_name||''} ${u.last_name||''}</span>
       </div>`;
  }
</script>
</body>
</html>
"#;
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
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
    #[serde(flatten)]
    rest: serde_json::Value, // keep any extra fields
}

#[post("/auth")]
async fn auth(
    cfg: web::Data<AppCfg>,
    body: web::Json<TgAuthPayload>,
) -> impl Responder {
    // Build data_check_string: sorted key=value lines, excluding "hash"
    // per https://core.telegram.org/widgets/login#checking-authorization
    let mut kv = BTreeMap::new();
    kv.insert("auth_date", body.auth_date.to_string());
    kv.insert("first_name", body.first_name.clone());
    if let Some(l) = &body.last_name { kv.insert("last_name", l.clone()); }
    kv.insert("id", body.id.to_string());
    if let Some(u) = &body.username { kv.insert("username", u.clone()); }
    if let Some(p) = &body.photo_url { kv.insert("photo_url", p.clone()); }

    let data_check_string = kv.iter()
        .map(|(k,v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    // secret_key = SHA256(bot_token)
    let secret_key = Sha256::digest(cfg.bot_token.as_bytes());
    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_key).unwrap();
    mac.update(data_check_string.as_bytes());
    let calc_hash = hex::encode(mac.finalize().into_bytes());

    if calc_hash != body.hash.to_lowercase() {
        return HttpResponse::Unauthorized().finish();
    }

    // OPTIONAL: check auth_date freshness (e.g., 1 day)
    let now = chrono::Utc::now().timestamp();
    if (now - body.auth_date).abs() > 86400 {
        return HttpResponse::Unauthorized().body("auth expired");
    }

    // Return only what we need to render
    let resp = serde_json::json!({
        "first_name": body.first_name,
        "last_name": body.last_name,
        "username": body.username,
        "photo_url": body.photo_url
    });
    HttpResponse::Ok().json(resp)
}

#[get("/widget-info")]
async fn widget_info(cfg: web::Data<AppCfg>) -> impl Responder {
    // Expose bot username for the widget; never expose the token.
    HttpResponse::Ok().json(serde_json::json!({
        "bot_username": cfg.bot_username
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);

    let cfg = AppCfg {
        bot_username: std::env::var("TELEGRAM_BOT_USERNAME").expect("TELEGRAM_BOT_USERNAME"),
        bot_token: std::env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN"),
    };

    println!("Starting server on http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cfg.clone()))
            .service(index)
            .service(auth)
            .service(widget_info)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
