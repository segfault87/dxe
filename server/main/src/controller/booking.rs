mod adhoc_reservation;
mod amend;
mod calendar;
mod cancel;
mod check;
mod doorlock;
mod get;
mod recording;
mod submit;

use actix_web::web;

pub fn bookings_scope() -> actix_web::Scope {
    web::scope("/bookings")
        .service(web::resource("/calendar").route(web::get().to(calendar::get)))
        .service(web::resource("/check").route(web::post().to(check::post)))
        .service(web::resource("").route(web::post().to(submit::post)))
}

pub fn booking_scope() -> actix_web::Scope {
    web::scope("/booking")
        .service(
            web::resource("/{booking_id}")
                .route(web::get().to(get::get))
                .route(web::delete().to(cancel::delete))
                .route(web::put().to(amend::put)),
        )
        .service(web::resource("/{booking_id}/open").route(web::post().to(doorlock::post)))
        .service(web::resource("/{booking_id}/recording").route(web::get().to(recording::get)))
}

pub fn adhoc_reservation_scope() -> actix_web::Scope {
    web::scope("/adhoc-reservation").service(
        web::resource("/{adhoc_reservation_id}").route(web::delete().to(adhoc_reservation::delete)),
    )
}
