mod handle_auth;
mod kakao_auth;
mod kakao_register;
mod logout;

use actix_web::web;

pub fn scope() -> actix_web::Scope {
    web::scope("/auth")
        .service(web::resource("/kakao/redirect").route(web::get().to(kakao_auth::redirect)))
        .service(web::resource("/kakao").route(web::post().to(kakao_register::post)))
        .service(web::resource("/logout").route(web::get().to(logout::redirect)))
        .service(web::resource("/login").route(web::post().to(handle_auth::post)))
}
