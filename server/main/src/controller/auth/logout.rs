use actix_web::HttpResponse;
use actix_web::body::BoxBody;
use actix_web::cookie::Cookie;
use actix_web::http::header::LOCATION;

pub async fn redirect() -> HttpResponse<BoxBody> {
    let mut access_token = Cookie::new("access_token", "");
    access_token.set_path("/");
    access_token.make_removal();
    let mut refresh_token = Cookie::new("refresh_token", "");
    refresh_token.set_path("/");
    refresh_token.make_removal();

    HttpResponse::PermanentRedirect()
        .insert_header((LOCATION, "/"))
        .cookie(access_token)
        .cookie(refresh_token)
        .body("logged out")
}
