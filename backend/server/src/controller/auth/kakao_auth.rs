use std::collections::HashSet;
use std::time::Duration;

use actix_jwt_auth_middleware::TokenSigner;
use actix_web::body::BoxBody;
use actix_web::cookie::Cookie;
use actix_web::cookie::time::OffsetDateTime;
use actix_web::http::header::LOCATION;
use actix_web::{HttpResponse, ResponseError, web};
use dxe_data::queries::user::{get_user_by_foreign_id, is_administrator};
use dxe_extern::kakao::client as kakao_client;
use dxe_extern::kakao::models::AccountPropertyKey;
use dxe_types::IdentityProvider;
use jwt_compact::alg::Ed25519;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::config::{KakaoAuthConfig, UrlConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::handlers::auth;
use crate::session::UserSession;
use crate::utils::aes::{AesCrypto, Error as AesError};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KakaoAuthState {
    redirect_to: Option<String>,
}

pub async fn redirect(
    now: Now,
    query: web::Query<auth::KakaoAuthRedirectQuery>,
    kakao_auth: web::Data<KakaoAuthConfig>,
    database: web::Data<SqlitePool>,
    token_signer: web::Data<TokenSigner<UserSession, Ed25519>>,
    aes_crypto: web::Data<AesCrypto>,
    url_config: web::Data<UrlConfig>,
) -> Result<HttpResponse<BoxBody>, Error> {
    if let Some(code) = &query.code {
        let redirect_to = if let Some(state) = query.state.as_ref()
            && let Ok(state) = serde_json::from_str::<KakaoAuthState>(state.as_str())
        {
            state.redirect_to
        } else {
            None
        }
        .unwrap_or("/".to_owned());

        let mut redirect_url = url_config.base_url.clone();
        redirect_url.set_path("/api/auth/kakao/redirect");

        let token =
            kakao_client::get_oauth_token(kakao_auth.get_ref(), code, redirect_url.as_str())
                .await?;

        let me = kakao_client::get_me(
            &token,
            HashSet::from([AccountPropertyKey::Profile, AccountPropertyKey::Email]),
        )
        .await?;

        let name = if let Some(account) = me.kakao_account.as_ref()
            && let Some(profile) = account.profile.as_ref()
        {
            profile.nickname.clone().unwrap_or_default()
        } else {
            String::new()
        };
        let foreign_id = me.id.to_string();

        let mut tx = database.begin().await.map_err(dxe_data::Error::Sqlx)?;
        let user =
            get_user_by_foreign_id(&mut tx, IdentityProvider::Kakao, foreign_id.as_str(), *now)
                .await?;

        if let Some(user) = user {
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

            Ok(HttpResponse::Found()
                .insert_header((LOCATION, redirect_to))
                .cookie(access_cookie)
                .cookie(refresh_cookie)
                .finish())
        } else {
            let encrypted_access_token = aes_crypto.encrypt(None, token.access_token.as_bytes())?;
            let mut cookie_bearer = Cookie::build("kakao_bearer_token", encrypted_access_token)
                .path("/")
                .expires(OffsetDateTime::now_utc() + Duration::from_secs(180))
                .http_only(true)
                .secure(true);

            if let Some(domain) = url_config.base_url.domain() {
                cookie_bearer = cookie_bearer.domain(domain);
            }

            Ok(HttpResponse::Found()
                .insert_header((
                    LOCATION,
                    format!(
                        "/register?name={}&redirect_to={}",
                        urlencoding::encode(&name),
                        redirect_to,
                    ),
                ))
                .cookie(cookie_bearer.finish())
                .finish())
        }
    } else if let Some(error) = &query.error {
        let error_message = query.error_description.clone().unwrap_or_default();
        Ok(HttpResponse::TemporaryRedirect()
            .insert_header((
                LOCATION,
                format!("/error?error={error}&message={error_message}"),
            ))
            .finish())
    } else {
        Ok(HttpResponse::TemporaryRedirect()
            .insert_header((LOCATION, "/".to_owned()))
            .finish())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kakao API error: {0}")]
    Kakao(#[from] kakao_client::Error),
    #[error("Data error: {0}")]
    Data(#[from] dxe_data::Error),
    #[error("Error generating/validating token: {0}")]
    Jwt(actix_jwt_auth_middleware::AuthError),
    #[error("Error encrypting/decrypting cookie data: {0}")]
    Aes(#[from] AesError),
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let message = format!("{self}");
        let url = format!("/error?message={}", urlencoding::encode(message.as_str()));

        HttpResponse::TemporaryRedirect()
            .insert_header((LOCATION, url))
            .finish()
    }
}
