mod booking;
mod user;

use actix_web::web;

pub fn api() -> actix_web::Scope {
    web::scope("/api").service(user::scope())
}
