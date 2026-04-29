use actix_web::web;

use crate::http::handlers::{
    auth,
    bookings,
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

    cfg.service(bookings::create_booking_request_handler);
    cfg.service(bookings::begin_booking_payment_handler);
    cfg.service(bookings::confirm_booking_payment_handler);

    cfg.service(experts::get_edit_expert_handler);
    cfg.service(experts::update_edit_expert_handler);
    cfg.service(experts::delete_calendar_connection_handler);
    cfg.service(experts::get_public_expert_handler);
    cfg.service(experts::get_popular_experts_handler);

    cfg.service(google_calendar::google_start);
    cfg.service(google_calendar::google_callback);
    cfg.service(google_calendar::google_session_get);
    cfg.service(google_calendar::google_session_select);

    cfg.service(pages::expert_public_page);
    cfg.service(pages::expert_edit_page);
}