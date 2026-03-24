use actix_web::web;

use crate::http::handlers::{auth, expert_setup, health};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health);
    cfg.service(auth::tg_auth);
    cfg.service(auth::link_wallet);
    cfg.service(expert_setup::register_expert_setup_handler);
}