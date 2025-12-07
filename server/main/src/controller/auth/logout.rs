use actix_web::body::BoxBody;
use actix_web::http::header::LOCATION;
use actix_web::{HttpResponse, web};

use crate::config::UrlConfig;
use crate::utils::session::log_out;

pub async fn redirect(url_config: web::Data<UrlConfig>) -> HttpResponse<BoxBody> {
    log_out(
        HttpResponse::PermanentRedirect().insert_header((LOCATION, "/")),
        &url_config,
    )
    .body("logged out")
}
