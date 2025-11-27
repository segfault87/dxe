use std::path::PathBuf;
use std::sync::Arc;

use gcp_auth::Token;
use reqwest::StatusCode;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;

pub const SCOPE: &str = "https://www.googleapis.com/auth/drive";

const UPLOAD_URL: &str = "https://www.googleapis.com/upload/drive/v3/files";
const PERMISSION_URL: &str = "https://www.googleapis.com/drive/v3/files/{id}/permissions";

pub trait GoogleDriveConfig {
    fn parent(&self) -> &str;
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UploadResult {
    id: String,
    web_view_link: url::Url,
}

#[derive(Debug, Clone)]
pub struct GoogleDriveClient {
    token: Arc<Token>,
    client: reqwest::Client,

    parent: String,
}

impl GoogleDriveClient {
    pub fn new(token: Arc<Token>, config: &impl GoogleDriveConfig) -> Self {
        Self {
            token,
            client: reqwest::Client::new(),

            parent: config.parent().to_owned(),
        }
    }

    pub async fn upload(&self, path: PathBuf, content_type: &str) -> Result<url::Url, Error> {
        let file_name = path
            .file_name()
            .ok_or(Error::InvalidFileName(path.clone()))?
            .to_str()
            .ok_or(Error::InvalidFileName(path.clone()))?
            .to_owned();

        let metadata = serde_json::json!({
            "parents": vec![self.parent.clone()],
            "name": file_name,
        });

        let form = Form::new()
            .part(
                "metadata",
                Part::text(serde_json::to_string(&metadata).unwrap())
                    .mime_str("application/json;charset=UTF-8")?,
            )
            .part("file", Part::file(path).await?.mime_str(content_type)?);

        let response = self
            .client
            .post(UPLOAD_URL)
            .query(&serde_json::json!({
                "uploadType": "multipart",
                "fields": "id,webViewLink",
                "supportsAllDrives": "true",
            }))
            .bearer_auth(self.token.as_str())
            .multipart(form)
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            let status = response.status();
            println!("Error response: {}", response.text().await?);
            return Err(Error::Status(status));
        }

        let UploadResult { id, web_view_link } = response.json().await?;

        let permission_payload = serde_json::json!({
            "type": "anyone",
            "role": "reader",
        });

        let response = self
            .client
            .post(PERMISSION_URL.replace("{id}", &id))
            .query(&serde_json::json!({
                "supportsAllDrives": "true",
            }))
            .json(&permission_payload)
            .bearer_auth(self.token.as_str())
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            let status = response.status();
            println!("{}", response.text().await?);

            return Err(Error::Status(status));
        }

        Ok(web_view_link)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Invalid file name: {0}")]
    InvalidFileName(PathBuf),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP status: {0}")]
    Status(StatusCode),
}
