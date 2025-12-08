mod payment_toss;

use actix_web::web;

pub fn scope() -> actix_web::Scope {
    web::scope("/payments")
        .service(web::resource("/toss").route(web::post().to(payment_toss::post)))
        .service(
            web::resource("/toss/confirm").route(web::post().to(payment_toss::confirm_payment)),
        )
        .service(
            web::resource("/toss/order/{foreign_payment_id}")
                .route(web::get().to(payment_toss::get))
                .route(web::delete().to(payment_toss::delete)),
        )
}
