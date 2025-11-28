pub mod models;

use std::sync::Arc;

use reqwest::{Client, StatusCode};
use tokio::sync::Mutex;

use crate::biztalk::models::{
    AlimTalkButtonAttachment, AllimTalkAttachment, GetTokenResponse, SendAlimTalkRequest,
    SendAlimTalkResponse,
};

const BIZTALK_RESPONSE_OK: &str = "1000";

const BIZTALK_GET_TOKEN_URL: &str = "https://www.biztalk-api.com/v2/auth/getToken";
const BIZTALK_SEND_ALIMTALK_URL: &str = "https://www.biztalk-api.com/v2/kko/sendAlimTalk";

pub trait BiztalkConfig {
    fn bs_id(&self) -> &str;
    fn password(&self) -> &str;
    fn sender_key(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct BiztalkClient {
    client: Client,
    token: Arc<Mutex<Option<String>>>,

    bs_id: String,
    password: String,
    sender_key: String,
}

impl BiztalkClient {
    pub fn new(config: &impl BiztalkConfig) -> Self {
        Self {
            client: Client::new(),
            token: Arc::new(Mutex::new(None)),

            bs_id: config.bs_id().to_owned(),
            password: config.password().to_owned(),
            sender_key: config.sender_key().to_owned(),
        }
    }

    async fn refresh_token(&self) -> Result<(), Error> {
        let request = self
            .client
            .post(BIZTALK_GET_TOKEN_URL)
            .json(&models::GetTokenRequest {
                bsid: self.bs_id.clone(),
                password: self.password.clone(),
            })
            .build()?;

        let response = self.client.execute(request).await?;
        let payload: GetTokenResponse = response.json().await?;

        if payload.response_code == BIZTALK_RESPONSE_OK
            && let Some(token) = payload.token
        {
            *self.token.lock().await = Some(token);
            Ok(())
        } else {
            Err(Error::Biztalk(
                payload.response_code,
                payload.message.clone().unwrap_or_default(),
            ))
        }
    }

    async fn post(&self, request: reqwest::RequestBuilder) -> Result<reqwest::Response, Error> {
        if self.token.lock().await.is_none() {
            self.refresh_token().await?;
        }

        let token = self.token.lock().await.clone().unwrap();
        let request = request.header("bt-token", token).build()?;

        let response = self.client.execute(request).await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            Err(Error::Unauthorized)
        } else {
            Ok(response)
        }
    }

    pub async fn send_alimtalk(
        &self,
        recipient: &str,
        template_code: &str,
        message: String,
        buttons: Option<Vec<AlimTalkButtonAttachment>>,
    ) -> Result<(), Error> {
        let request = self
            .client
            .post(BIZTALK_SEND_ALIMTALK_URL)
            .json(&SendAlimTalkRequest {
                msg_idx: uuid::Uuid::new_v4().to_string(),
                app_user_id: recipient.to_owned(),
                country_code: "82".to_string(),
                sender_key: self.sender_key.clone(),
                template_code: template_code.to_owned(),
                res_method: Default::default(),
                message,
                attach: buttons.map(|buttons| AllimTalkAttachment {
                    button: Some(buttons),
                }),
            });

        let result = {
            let result = self.post(request.try_clone().unwrap()).await;
            if matches!(result, Err(Error::Unauthorized)) {
                self.refresh_token().await?;

                self.post(request).await
            } else {
                result
            }
        }?;

        let response: SendAlimTalkResponse = result.json().await?;

        if response.response_code == BIZTALK_RESPONSE_OK {
            Ok(())
        } else {
            Err(Error::Biztalk(response.response_code, response.message))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Error parsing JSON response: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Token is empty.")]
    NoToken,
    #[error("Error from Biztalk: {1}")]
    Biztalk(String, String),
    #[error("Unauthorized")]
    Unauthorized,
}
