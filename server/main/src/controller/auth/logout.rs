use actix_web::body::BoxBody;
use actix_web::cookie::Cookie;
use actix_web::http::Uri;
use actix_web::http::header::LOCATION;
use actix_web::{HttpResponse, web};

use crate::config::UrlConfig;

pub async fn redirect(url_config: web::Data<UrlConfig>) -> HttpResponse<BoxBody> {
    let mut access_token = Cookie::new("access_token", "");
    access_token.set_path("/");
    access_token.set_http_only(true);
    access_token.make_removal();
    let mut refresh_token = Cookie::new("refresh_token", "");
    refresh_token.set_path("/");
    refresh_token.set_http_only(true);
    refresh_token.make_removal();

    let base_url = Uri::try_from(&url_config.base_url).unwrap();
    if let Some(host) = base_url.host() {
        access_token.set_domain(host);
        refresh_token.set_domain(host);
    }

    HttpResponse::PermanentRedirect()
        .insert_header((LOCATION, "/"))
        .cookie(access_token)
        .cookie(refresh_token)
        .body("logged out")
}
