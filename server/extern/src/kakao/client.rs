use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::models::{AccountPropertyKey, KakaoAccount, Partner};
use super::{BearerToken, BearerTokenExt, KakaoRestApiConfig};

const KAKAO_OAUTH_TOKEN_URL: &str = "https://kauth.kakao.com/oauth/token";
const KAKAO_USER_ME_URL: &str = "https://kapi.kakao.com/v2/user/me";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Kakao(#[from] KakaoError),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize)]
pub struct KakaoError {
    pub error: String,
    pub error_description: String,
    pub error_code: String,
}

impl std::error::Error for KakaoError {}

impl std::fmt::Display for KakaoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_description)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    Error(KakaoError),
    Response(T),
}

impl<T> From<Response<T>> for Result<T, Error> {
    fn from(value: Response<T>) -> Self {
        match value {
            Response::Response(value) => Ok(value),
            Response::Error(e) => Err(e.into()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OAuthTokenResponse {
    pub token_type: String,
    pub access_token: String,
    pub id_token: Option<String>,
    pub expires_in: i64,
    pub refresh_token: String,
    pub refresh_token_expires_in: i64,
    pub scope: Option<String>,
}

impl BearerToken for OAuthTokenResponse {
    fn access_token(&self) -> &str {
        &self.access_token
    }
}

pub async fn get_oauth_token(
    config: &impl KakaoRestApiConfig,
    code: &str,
    redirect_url: &str,
) -> Result<OAuthTokenResponse, Error> {
    let form = HashMap::from([
        ("grant_type", "authorization_code"),
        ("client_id", config.client_id()),
        ("client_secret", config.auth_client_secret()),
        ("redirect_uri", redirect_url),
        ("code", code),
    ]);

    reqwest::Client::new()
        .post(KAKAO_OAUTH_TOKEN_URL)
        .form(&form)
        .send()
        .await?
        .json::<Response<OAuthTokenResponse>>()
        .await?
        .into()
}

#[derive(Debug, Deserialize)]
pub struct UserMeResponse {
    pub id: i64,
    pub has_signed_up: Option<bool>,
    pub connected_at: Option<DateTime<Utc>>,
    pub synched_at: Option<DateTime<Utc>>,
    pub properties: Option<HashSet<AccountPropertyKey>>,
    pub kakao_account: Option<KakaoAccount>,
    pub for_partner: Option<Partner>,
}

pub async fn get_me(
    bearer_token: &impl BearerToken,
    property_keys: HashSet<AccountPropertyKey>,
) -> Result<UserMeResponse, Error> {
    let form = HashMap::from([
        ("secure_resource", "true".to_owned()),
        (
            "property_keys",
            serde_json::to_string(&property_keys).unwrap(),
        ),
    ]);

    let (key, value) = bearer_token.header();

    reqwest::Client::new()
        .post(KAKAO_USER_ME_URL)
        .form(&form)
        .header(key, value)
        .send()
        .await?
        .json::<Response<UserMeResponse>>()
        .await?
        .into()
}
