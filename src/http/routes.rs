use actix_web::web;

use crate::http::handlers::{auth, health};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health);
    cfg.service(auth::tg_auth);
    cfg.service(auth::link_wallet);
}