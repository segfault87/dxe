mod booking;
mod bookings;
mod groups;
mod reservation;
mod users;

use actix_web::web;

pub fn scope() -> actix_web::Scope {
    web::scope("/admin")
        .service(web::resource("/booking/{booking_id}").route(web::put().to(booking::put)))
        .service(web::resource("/bookings").route(web::get().to(bookings::get)))
        .service(
            web::resource("/reservations")
                .route(web::get().to(reservation::get))
                .route(web::post().to(reservation::post)),
        )
        .service(
            web::resource("/reservation/{reservation_id}")
                .route(web::delete().to(reservation::delete)),
        )
        .service(web::resource("/users").route(web::get().to(users::get)))
        .service(web::resource("/groups").route(web::get().to(groups::get)))
}
