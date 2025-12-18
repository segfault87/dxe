mod group;
mod groups;
mod me;

use actix_web::web;

pub fn scope() -> actix_web::Scope {
    web::scope("/user")
        .service(
            web::resource("/me")
                .route(web::get().to(me::get))
                .route(web::post().to(me::post)),
        )
        .service(
            web::resource("/group/{group_id}")
                .route(web::get().to(group::get))
                .route(web::put().to(group::put))
                .route(web::delete().to(group::delete)),
        )
        .service(
            web::resource("/group/{group_id}/membership")
                .route(web::put().to(group::membership_put))
                .route(web::delete().to(group::membership_delete)),
        )
        .service(
            web::resource("/groups")
                .route(web::get().to(groups::get))
                .route(web::post().to(groups::post)),
        )
}
