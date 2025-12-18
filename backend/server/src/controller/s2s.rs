mod adhoc_parkings;
mod audio_recording;
mod booking_reminder;
mod bookings;
mod doorlock;
mod telemetry;
mod units;

use actix_web::web;

pub fn scope() -> actix_web::Scope {
    web::scope("/s2s")
        .service(web::resource("/pending-bookings").route(web::get().to(bookings::get)))
        .service(web::resource("/units").route(web::get().to(units::get)))
        .service(
            web::resource("/booking/{booking_id}/recording")
                .route(web::post().to(audio_recording::post)),
        )
        .service(
            web::resource("/booking/{booking_id}/telemetry").route(web::post().to(telemetry::post)),
        )
        .service(
            web::resource("/booking/{booking_id}/reminder")
                .route(web::post().to(booking_reminder::post)),
        )
        .service(web::resource("/adhoc-parkings").route(web::get().to(adhoc_parkings::get)))
        .service(web::resource("/doorlock").route(web::post().to(doorlock::post)))
}
