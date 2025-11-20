use serde::{Deserialize, Serialize};

const ITSOKEY_API_ENDPOINT: &str = "https://v2.api.itsokey.kr/api/device/permission/control.do";

pub trait ItsokeyConfig {
    fn device_id(&self) -> &str;
    fn dp_id(&self) -> &str;
    fn password(&self) -> &str;
}

#[derive(Clone, Debug)]
pub struct ItsokeyClient {
    client: reqwest::Client,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ControlBehavior {
    Open,
    Close,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItsokeyControlRequest {
    device_idx: String,
    dp_idx: String,
    password: String,
    r#type: ControlBehavior,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItsokeyControlResponse {
    data: serde_json::Value,
    result: bool,
    code: i32,
    message: String,
}

impl ItsokeyClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn open(&self, config: &impl ItsokeyConfig) -> Result<(), Error> {
        let request = self
            .client
            .post(ITSOKEY_API_ENDPOINT)
            .json(&ItsokeyControlRequest {
                device_idx: config.device_id().to_owned(),
                dp_idx: config.dp_id().to_owned(),
                password: config.password().to_owned(),
                r#type: ControlBehavior::Open,
            })
            .build()?;

        let response: ItsokeyControlResponse = self.client.execute(request).await?.json().await?;
        if !response.result {
            Err(Error::Itsokey(response.message))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Error parsing response: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Error executing command: {0}")]
    Itsokey(String),
}
