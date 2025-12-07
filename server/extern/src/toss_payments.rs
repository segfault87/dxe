pub mod handlers;
pub mod types;

use std::fmt::Display;

use reqwest::StatusCode;

const TOSS_PAYMENTS_CONFIRM_URL: &str = "https://api.tosspayments.com/v1/payments/confirm";
const TOSS_PAYMENTS_CANCEL_URL: &str =
    "https://api.tosspayments.com/v1/payments/{payment_key}/cancel";

pub trait TossPaymentsConfig {
    fn server_secret_key(&self) -> &str;
}

pub struct TossPaymentsClient {
    client: reqwest::Client,

    server_secret_key: String,
}

impl TossPaymentsClient {
    pub fn new(config: &impl TossPaymentsConfig) -> Self {
        Self {
            client: reqwest::Client::new(),

            server_secret_key: config.server_secret_key().to_owned(),
        }
    }

    pub async fn confirm_payment<T: Display>(
        &self,
        order_id: &T,
        amount: i64,
        payment_key: &str,
    ) -> Result<types::Payment, Error> {
        let request = handlers::ConfirmPaymentRequest {
            order_id: format!("{order_id}"),
            amount,
            payment_key: payment_key.to_owned(),
        };

        let response = self
            .client
            .post(TOSS_PAYMENTS_CONFIRM_URL)
            .basic_auth::<String, String>(self.server_secret_key.clone(), None)
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if status == StatusCode::OK {
            Ok(response.json().await?)
        } else if let Ok(v) = response.json::<types::ErrorV1>().await {
            Err(v.into())
        } else {
            Err(Error::RemoteStatus(status))
        }
    }

    pub async fn cancel_payment(
        &self,
        payment_key: &str,
        cancel_reason: &str,
        cancel_amount: Option<i64>,
    ) -> Result<types::Payment, Error> {
        let url = TOSS_PAYMENTS_CANCEL_URL.replace("{payment_key}", payment_key);

        let request = handlers::CancelPaymentRequest {
            cancel_reason: cancel_reason.to_owned(),
            cancel_amount,
            tax_free_amount: None,
            currency: if cancel_amount.is_some() {
                Some(String::from("KRW"))
            } else {
                None
            },
            refund_receive_account: None,
        };

        let response = self
            .client
            .post(url)
            .basic_auth::<String, String>(self.server_secret_key.clone(), None)
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if status == StatusCode::OK {
            Ok(response.json().await?)
        } else if let Ok(v) = response.json::<types::ErrorV1>().await {
            Err(v.into())
        } else {
            Err(Error::RemoteStatus(status))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Error deserializing response: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Toss API invocation failed: {message} ({code})")]
    Remote { code: String, message: String },
    #[error("Unrecognized response from remote: {0}")]
    RemoteStatus(StatusCode),
}

impl From<types::ErrorV1> for Error {
    fn from(value: types::ErrorV1) -> Self {
        Self::Remote {
            code: value.code,
            message: value.message,
        }
    }
}
