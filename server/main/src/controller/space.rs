mod audio_recording;
mod bookings;
mod units;

use actix_web::web;

pub fn scope() -> actix_web::Scope {
    web::scope("/s2s")
        .service(web::resource("/pending-bookings").route(web::get().to(bookings::get)))
        .service(web::resource("/units").route(web::get().to(units::get)))
        .service(
            web::resource("/booking/{booking_id}/audio")
                .route(web::post().to(audio_recording::post)),
        )
}
