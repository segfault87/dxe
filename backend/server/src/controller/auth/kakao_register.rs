use actix_jwt_auth_middleware::TokenSigner;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use dxe_data::queries::user::create_user;
use dxe_extern::kakao::{BearerToken, client as kakao_client};
use dxe_types::IdentityProvider;
use jwt_compact::alg::Ed25519;
use sqlx::SqlitePool;

use crate::middleware::datetime_injector::Now;
use crate::models::Error;
use crate::models::handlers::auth;
use crate::services::telemetry::{NotificationSender, Priority};
use crate::session::UserSession;
use crate::utils::aes::AesCrypto;

struct SimpleBearerToken(String);

impl BearerToken for SimpleBearerToken {
    fn access_token(&self) -> &str {
        &self.0
    }
}

pub async fn post(
    now: Now,
    request: HttpRequest,
    body: web::Json<auth::KakaoAuthRegisterRequest>,
    database: web::Data<SqlitePool>,
    token_signer: web::Data<TokenSigner<UserSession, Ed25519>>,
    aes_crypto: web::Data<AesCrypto>,
    notification_sender: web::Data<NotificationSender>,
) -> Result<impl Responder, Error> {
    let cookie = request
        .cookie("kakao_bearer_token")
        .ok_or(Error::InvalidKakaoAccessToken)?;
    let encrypted_token = cookie.value();
    let token = aes_crypto
        .decrypt(None, encrypted_token.as_bytes())
        .map_err(|_| Error::InvalidKakaoAccessToken)?;
    let bearer_token = SimpleBearerToken(token);

    let me = kakao_client::get_me(&bearer_token, Default::default()).await?;

    let mut tx = database.begin().await?;

    let user_id = create_user(
        &mut tx,
        *now,
        IdentityProvider::Kakao,
        me.id.to_string().as_str(),
        body.name.as_str(),
        if body
            .license_plate_number
            .as_ref()
            .map(String::len)
            .unwrap_or_default()
            > 0
        {
            body.license_plate_number.as_deref()
        } else {
            None
        },
    )
    .await?;

    tx.commit().await.unwrap();

    notification_sender.enqueue(
        Priority::Low,
        format!("New member joined via Kakao: {}", body.name),
    );

    let session = UserSession {
        user_id,
        is_administrator: false,
    };

    let mut access_cookie = token_signer.create_access_cookie(&session)?;
    let mut refresh_cookie = token_signer.create_refresh_cookie(&session)?;

    access_cookie.set_path("/");
    refresh_cookie.set_path("/");

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(serde_json::json!({})))
}
