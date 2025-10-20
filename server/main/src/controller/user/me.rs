use actix_web::{HttpResponse, body::BoxBody, http::StatusCode};

pub async fn get() -> HttpResponse {
    HttpResponse::with_body(StatusCode::OK, BoxBody::new("Hello world!"))
}
