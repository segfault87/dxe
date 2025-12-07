use actix_jwt_auth_middleware::TokenSigner;
use actix_web::body::BoxBody;
use actix_web::{HttpResponse, web};
use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use dxe_data::queries::identity::get_user_plain_credential_with_handle;
use dxe_data::queries::user::is_administrator;
use jwt_compact::alg::Ed25519;
use sqlx::SqlitePool;

use crate::config::UrlConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::Error;
use crate::models::handlers::auth::{HandleAuthQuery, HandleAuthRequest, HandleAuthResponse};
use crate::session::UserSession;

pub async fn post(
    now: Now,
    query: web::Query<HandleAuthQuery>,
    body: web::Json<HandleAuthRequest>,
    database: web::Data<SqlitePool>,
    token_signer: web::Data<TokenSigner<UserSession, Ed25519>>,
    url_config: web::Data<UrlConfig>,
) -> Result<HttpResponse<BoxBody>, Error> {
    let mut tx = database.begin().await?;

    let (user, cred) = get_user_plain_credential_with_handle(&mut tx, &now, &body.handle)
        .await?
        .ok_or(Error::AuthFailed)?;

    let argon2 = Argon2::default();
    let hash = PasswordHash::new(&cred.argon2_password).map_err(|_| Error::AuthFailed)?;

    argon2
        .verify_password(body.password.as_bytes(), &hash)
        .map_err(|_| Error::AuthFailed)?;

    let session = UserSession {
        user_id: user.id,
        is_administrator: is_administrator(&mut tx, &user.id).await?,
    };

    let mut access_cookie = token_signer
        .create_access_cookie(&session)
        .map_err(Error::Jwt)?;
    let mut refresh_cookie = token_signer
        .create_refresh_cookie(&session)
        .map_err(Error::Jwt)?;

    access_cookie.set_http_only(true);
    access_cookie.set_path("/");
    refresh_cookie.set_http_only(true);
    refresh_cookie.set_path("/");

    if let Some(domain) = url_config.base_url.domain() {
        access_cookie.set_domain(domain);
        refresh_cookie.set_domain(domain);
    }

    let redirect_to = query.redirect_to.clone().unwrap_or(String::from("/"));

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(HandleAuthResponse { redirect_to }))
}
