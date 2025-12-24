use actix_web::HttpResponseBuilder;
use actix_web::cookie::Cookie;

use crate::config::UrlConfig;

pub fn log_out<'a>(
    response: &'a mut HttpResponseBuilder,
    url_config: &UrlConfig,
) -> &'a mut HttpResponseBuilder {
    let mut access_token = Cookie::new("_dxe_access_token", "");
    access_token.set_path("/");
    access_token.set_secure(true);
    access_token.set_http_only(true);
    access_token.make_removal();
    let mut refresh_token = Cookie::new("_dxe_refresh_token", "");
    refresh_token.set_path("/");
    refresh_token.set_secure(true);
    refresh_token.set_http_only(true);
    refresh_token.make_removal();

    if let Some(domain) = url_config.base_url.domain() {
        access_token.set_domain(domain);
        refresh_token.set_domain(domain);
    }

    response.cookie(access_token).cookie(refresh_token)
}
