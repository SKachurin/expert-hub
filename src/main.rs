mod config;
mod db;
mod entities;
mod http;
mod services;
mod state;

use actix_files::Files;
use actix_web::{middleware::NormalizePath, web, App, HttpServer};

use config::AppConfig;
use db::connect_db;
use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = AppConfig::from_env();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = connect_db(&database_url)
        .await
        .expect("failed to connect to database");

    let state = web::Data::new(AppState::new(config.clone(), db));

    HttpServer::new(move || {
        App::new()
            .wrap(NormalizePath::trim())
            .app_data(state.clone())
            .configure(http::routes::configure)
            .service(Files::new("/", "./public").index_file("index.html"))
    })
    .bind((config.host.clone(), config.port))?
    .run()
    .await
}