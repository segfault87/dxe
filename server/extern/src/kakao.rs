pub mod client;
pub mod models;

pub trait KakaoRestApiConfig {
    fn client_id(&self) -> &str;
    fn auth_client_secret(&self) -> &str;
}

pub trait BearerToken {
    fn access_token(&self) -> &str;
}
