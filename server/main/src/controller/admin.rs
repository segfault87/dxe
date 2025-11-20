mod booking;
mod pending_bookings;
mod reservation;

use actix_web::web;

pub fn scope() -> actix_web::Scope {
    web::scope("/admin")
        .service(web::resource("/bookings/pending").route(web::get().to(pending_bookings::get)))
        .service(web::resource("/booking/{booking_id}").route(web::put().to(booking::put)))
        .service(
            web::resource("/reservations")
                .route(web::get().to(reservation::get))
                .route(web::post().to(reservation::post)),
        )
        .service(
            web::resource("/reservation/{reservation_id}")
                .route(web::delete().to(reservation::delete)),
        )
}
