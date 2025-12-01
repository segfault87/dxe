mod adhoc_reservations;
mod booking;
mod bookings;
mod groups;
mod users;

use actix_web::web;

pub fn scope() -> actix_web::Scope {
    web::scope("/admin")
        .service(web::resource("/booking/{booking_id}").route(web::put().to(booking::put)))
        .service(web::resource("/bookings").route(web::get().to(bookings::get)))
        .service(
            web::resource("/adhoc-reservations")
                .route(web::get().to(adhoc_reservations::get))
                .route(web::post().to(adhoc_reservations::post)),
        )
        .service(
            web::resource("/adhoc-reservation/{reservation_id}")
                .route(web::delete().to(adhoc_reservations::delete)),
        )
        .service(web::resource("/users").route(web::get().to(users::get)))
        .service(web::resource("/groups").route(web::get().to(groups::get)))
}
