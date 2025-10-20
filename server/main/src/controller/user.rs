mod me;

use actix_web::web;

pub fn scope() -> actix_web::Scope {
    web::scope("/user").service(web::resource("/me").route(web::get().to(me::get)))
}
