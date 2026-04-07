use actix_web::web;

use crate::http::handlers::{
    auth,
    expert_setup,
    experts,
    google_calendar,
    health,
    pages,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health);

    cfg.service(auth::tg_auth);
    cfg.service(auth::link_wallet);

    cfg.service(expert_setup::register_expert_setup_handler);

    cfg.service(experts::get_expert_edit_handler);
    cfg.service(experts::update_expert_edit_handler);
    cfg.service(experts::delete_calendar_connection_handler);

    cfg.service(google_calendar::google_start);
    cfg.service(google_calendar::google_callback);
    cfg.service(google_calendar::google_session_get);
    cfg.service(google_calendar::google_session_select);

    cfg.service(pages::expert_edit_page);
}