mod pending_bookings;

use actix_web::web;

pub fn scope() -> actix_web::Scope {
    web::scope("/s2s")
        .service(web::resource("/pending-bookings").route(web::get().to(pending_bookings::get)))
}
