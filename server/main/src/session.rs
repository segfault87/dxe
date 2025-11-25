use actix_jwt_auth_middleware::FromRequest;
use dxe_types::UserId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, FromRequest)]
pub struct UserSession {
    pub user_id: UserId,
    pub is_administrator: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, FromRequest)]
pub struct TemporaryKakaoOauthCredential {
    pub token: dxe_extern::kakao::client::OAuthTokenResponse,
}
