use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(super) struct GetTokenRequest {
    pub bsid: String,
    #[serde(rename = "passwd")]
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(super) struct GetTokenResponse {
    pub response_code: String,
    pub token: Option<String>,
    #[serde(rename = "msg")]
    pub message: Option<String>,
    pub expire_date: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(super) enum SendMethod {
    Push,
}

impl Default for SendMethod {
    fn default() -> Self {
        Self::Push
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ButtonType {
    Wl,
}

impl Default for ButtonType {
    fn default() -> Self {
        Self::Wl
    }
}

#[derive(Debug, Serialize)]
pub struct AlimTalkButtonAttachment {
    pub name: String,
    pub r#type: ButtonType,
    pub url_mobile: String,
    pub url_pc: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AllimTalkAttachment {
    pub button: Option<Vec<AlimTalkButtonAttachment>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SendAlimTalkRequest {
    pub msg_idx: String,
    pub app_user_id: String,
    pub country_code: String,
    pub sender_key: String,
    #[serde(rename = "tmpltCode")]
    pub template_code: String,
    pub res_method: SendMethod,
    pub message: String,
    pub attach: Option<AllimTalkAttachment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SendAlimTalkResponse {
    pub response_code: String,
    #[serde(rename = "msg")]
    pub message: String,
}
