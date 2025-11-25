use base64::prelude::*;
use chrono::{TimeDelta, Utc};
use dxe_types::SpaceId;
use ed25519_compact::{Noise, SecretKey};
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use url::Url;

#[derive(Deserialize)]
pub struct TimestampResponse {
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct DxeClient {
    client: reqwest::Client,
    space_id: SpaceId,
    private_key: SecretKey,
    url_base: Url,
    server_clock_delta: TimeDelta,
    request_expires_in: TimeDelta,
}

impl DxeClient {
    pub fn new(
        space_id: SpaceId,
        url_base: Url,
        request_expires_in: TimeDelta,
        private_key: &[u8],
    ) -> Result<Self, Error> {
        let private_key = SecretKey::from_slice(private_key)?;

        Ok(Self {
            client: reqwest::Client::new(),
            space_id,
            private_key,
            url_base,
            server_clock_delta: TimeDelta::default(),
            request_expires_in,
        })
    }

    pub async fn synchronize_clock(&mut self) -> Result<(), Error> {
        let mut url = self.url_base.clone();
        url.set_path("/api/timestamp");

        let mut deltas = vec![];

        for _ in 0..5 {
            let now = Utc::now().timestamp_millis();
            let request = self.client.get(url.clone()).send().await?;
            let response: TimestampResponse = request.json().await?;

            let delta = response.timestamp - now;
            deltas.push(delta);
        }

        let delta = deltas
            .iter()
            .min_by(|a, b| a.abs().cmp(&b.abs()))
            .cloned()
            .unwrap_or_default()
            / 2;

        self.server_clock_delta = TimeDelta::milliseconds(delta);

        Ok(())
    }

    fn make_signature_body(
        method: Method,
        path: &str,
        query: Option<&str>,
        expires_in: &str,
        body: Option<&str>,
    ) -> Vec<u8> {
        let mut result = vec![];

        result.extend(expires_in.as_bytes());
        result.extend(method.as_str().as_bytes());
        result.extend(path.as_bytes());
        if let Some(query) = query {
            result.extend(query.as_bytes());
        }
        if let Some(body) = body {
            result.extend(body.as_bytes());
        }

        result
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        query: Option<&str>,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, Error> {
        let expires_in = (Utc::now() + self.server_clock_delta + self.request_expires_in)
            .timestamp_millis()
            .to_string();

        let path = format!("/api/s2s{path}");

        let mut url = self.url_base.clone();
        url.set_path(&path);
        url.set_query(query);

        let body = body.map(|v| v.to_string());

        let signature = self.private_key.sign(
            Self::make_signature_body(
                Method::GET,
                &path,
                query,
                expires_in.as_str(),
                body.as_ref().map(|v| v.as_str()),
            ),
            Some(Noise::generate()),
        );

        let signature_base64 = BASE64_STANDARD.encode(signature);

        let mut request = self
            .client
            .request(method, url)
            .header("X-Signature-Expires-In", expires_in)
            .header("X-Signature", signature_base64)
            .header("X-Space-Id", self.space_id.to_string());

        if let Some(body) = body {
            request = request
                .header("Content-Type", "application/json")
                .body(body);
        }

        let response = request.send().await?;

        let status = response.status();
        let json: serde_json::Value = response.json().await?;
        if status != StatusCode::OK {
            if let Some(object) = json.as_object()
                && let Some(r#type) = object.get("type")
                && let Some(message) = object.get("message")
            {
                Err(Error::Remote {
                    r#type: r#type.to_string(),
                    message: message.to_string(),
                })
            } else {
                Err(Error::Status(status))
            }
        } else {
            Ok(json)
        }
    }

    pub async fn get<R: DeserializeOwned>(
        &self,
        path: &str,
        query: Option<&str>,
    ) -> Result<R, Error> {
        let response = self.request(Method::GET, path, query, None).await?;

        Ok(serde_json::from_value(response)?)
    }

    pub async fn delete<R: DeserializeOwned>(
        &self,
        path: &str,
        query: Option<&str>,
    ) -> Result<R, Error> {
        let response = self.request(Method::DELETE, path, query, None).await?;

        Ok(serde_json::from_value(response)?)
    }

    pub async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        query: Option<&str>,
        body: T,
    ) -> Result<R, Error> {
        let response = self
            .request(Method::POST, path, query, Some(serde_json::to_value(body)?))
            .await?;

        Ok(serde_json::from_value(response)?)
    }

    pub async fn put<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        query: Option<&str>,
        body: T,
    ) -> Result<R, Error> {
        let response = self
            .request(Method::PUT, path, query, Some(serde_json::to_value(body)?))
            .await?;

        Ok(serde_json::from_value(response)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[cfg_attr(debug_assertions, error("Crypto error: {0}"))]
    #[cfg_attr(not(debug_assertions), error("Crypto error"))]
    Crypto(#[from] ed25519_compact::Error),
    #[error("Invalid url format: {0}")]
    Url(#[from] url::ParseError),
    #[error("Could not serialize data: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("API call failed: {0}")]
    Status(StatusCode),
    #[error("API call failed: {message} ({type})")]
    Remote { message: String, r#type: String },
}
