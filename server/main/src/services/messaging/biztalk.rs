use dxe_data::types::BookingId;
use dxe_extern::biztalk::models::AlimTalkButtonAttachment;

use super::MessagingBackend;
use crate::config::BiztalkConfig;

const MESSAGE_RESERVATION_CANCEL_CONFIRM_01: &str =
    include_str!("biztalk/RESERVATION_CANCEL_CONFIRM_01.txt").trim_ascii();
const MESSAGE_RESERVATION_CANCEL_HRF_01: &str =
    include_str!("biztalk/RESERVATION_CANCEL_HRF_01.txt").trim_ascii();
const MESSAGE_RESERVATION_CANCEL_NRF_01: &str =
    include_str!("biztalk/RESERVATION_CANCEL_NRF_01.txt").trim_ascii();
const MESSAGE_RESERVATION_CANCEL_RF_01: &str =
    include_str!("biztalk/RESERVATION_CANCEL_RF_01.txt").trim_ascii();
const MESSAGE_RESERVATION_CONFIRMATION_01: &str =
    include_str!("biztalk/RESERVATION_CONFIRMATION_01.txt").trim_ascii();

const TEMPLATE_RESERVATION_CANCEL_CONFIRM: &str = "RESERVATION_CONFIRM_01";
const TEMPLATE_RESERVATION_CANCEL_NO_REFUND: &str = "RESERVATION_CANCEL_NRF_01";
const TEMPLATE_RESERVATION_CANCEL_HALF_REFUND: &str = "RESERVATION_CANCEL_HRF_01";
const TEMPLATE_RESERVATION_CANCEL_FULL_REFUND: &str = "RESERVATION_CANCEL_RF_01";
const TEMPLATE_RESERVATION_CONFIRMATION: &str = "RESERVATION_CONFIRMATION_01";

pub type BiztalkRecipient = String;
pub type BiztalkSender = super::MessagingSender<BiztalkRecipient>;

#[derive(Debug, Clone)]
pub struct BiztalkClient {
    client: dxe_extern::biztalk::BiztalkClient,
}

impl BiztalkClient {
    pub fn new(config: &BiztalkConfig) -> Self {
        Self {
            client: dxe_extern::biztalk::BiztalkClient::new(config),
        }
    }
}

#[async_trait::async_trait]
impl MessagingBackend for BiztalkClient {
    type Recipient = BiztalkRecipient;
    type Error = Error;

    async fn send_booking_confirmation(
        &self,
        recipients: Vec<Self::Recipient>,
        booking_id: &BookingId,
        customer_name: &str,
        reservation_time: &str,
    ) -> Result<(), Self::Error> {
        let message = MESSAGE_RESERVATION_CONFIRMATION_01
            .replace("#{customer}", customer_name)
            .replace("#{reservation_dt}", reservation_time);

        let url_mobile = format!("https://dream-house.kr/reservation/{booking_id}");

        let mut error = None;

        for recipient in recipients {
            if let Err(e) = self
                .client
                .send_alimtalk(
                    &recipient,
                    TEMPLATE_RESERVATION_CONFIRMATION,
                    message.clone(),
                    Some(vec![AlimTalkButtonAttachment {
                        name: "이용 안내".to_owned(),
                        r#type: Default::default(),
                        url_mobile: url_mobile.clone(),
                    }]),
                )
                .await
            {
                error = Some(e);
            }
        }

        if let Some(error) = error {
            Err(error.into())
        } else {
            Ok(())
        }
    }

    async fn send_cancel_notification(
        &self,
        recipients: Vec<Self::Recipient>,
        _booking_id: &BookingId,
        customer_name: &str,
        reservation_time: &str,
        refund_rate: i32,
    ) -> Result<(), Self::Error> {
        let (template, message) = match refund_rate {
            0 => (
                TEMPLATE_RESERVATION_CANCEL_NO_REFUND,
                MESSAGE_RESERVATION_CANCEL_NRF_01
                    .replace("#{customer}", customer_name)
                    .replace("#{reservation_dt}", reservation_time),
            ),
            50 => (
                TEMPLATE_RESERVATION_CANCEL_HALF_REFUND,
                MESSAGE_RESERVATION_CANCEL_HRF_01
                    .replace("#{customer}", customer_name)
                    .replace("#{reservation_dt}", reservation_time),
            ),
            100 => (
                TEMPLATE_RESERVATION_CANCEL_FULL_REFUND,
                MESSAGE_RESERVATION_CANCEL_RF_01
                    .replace("#{customer}", customer_name)
                    .replace("#{reservation_dt}", reservation_time),
            ),
            _ => {
                return Err(Error::NoTemplateForRefundRate(refund_rate));
            }
        };

        let mut error = None;

        for recipient in recipients {
            if let Err(e) = self
                .client
                .send_alimtalk(&recipient, template, message.clone(), None)
                .await
            {
                error = Some(e);
            }
        }

        if let Some(error) = error {
            Err(error.into())
        } else {
            Ok(())
        }
    }

    async fn send_refund_confirmation(
        &self,
        recipient: Self::Recipient,
        _booking_id: &BookingId,
        customer_name: &str,
        reservation_time: &str,
        refunded_price: i64,
    ) -> Result<(), Self::Error> {
        let message = MESSAGE_RESERVATION_CANCEL_CONFIRM_01
            .replace("#{customer}", customer_name)
            .replace("#{reservation_dt}", reservation_time)
            .replace("#{refund_price}", &refunded_price.to_string());

        self.client
            .send_alimtalk(
                &recipient,
                TEMPLATE_RESERVATION_CANCEL_CONFIRM,
                message.clone(),
                None,
            )
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No template for refund rate: {0}")]
    NoTemplateForRefundRate(i32),
    #[error("Biztalk Error: {0}")]
    Biztalk(#[from] dxe_extern::biztalk::Error),
}
