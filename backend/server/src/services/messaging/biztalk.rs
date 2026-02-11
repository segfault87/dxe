use chrono::{DateTime, Utc};
use dxe_data::entities::{AudioRecording, Booking};
use dxe_extern::biztalk::models::AlimTalkButtonAttachment;

use super::MessagingBackend;
use crate::config::{BiztalkConfig, TimeZoneConfig, UrlConfig};

const MESSAGE_AUDIO_READY: &str = include_str!("biztalk/AUDIO_READY_01.txt").trim_ascii();
const MESSAGE_RESERVATION_CANCEL_CONFIRM: &str =
    include_str!("biztalk/RESERVATION_CANCEL_CONFIRM_01.txt").trim_ascii();
const MESSAGE_RESERVATION_CANCEL_HRF: &str =
    include_str!("biztalk/RESERVATION_CANCEL_HRF_01.txt").trim_ascii();
const MESSAGE_RESERVATION_CANCEL_NRF: &str =
    include_str!("biztalk/RESERVATION_CANCEL_NRF_01.txt").trim_ascii();
const MESSAGE_RESERVATION_CANCEL_RF: &str =
    include_str!("biztalk/RESERVATION_CANCEL_RF_01.txt").trim_ascii();
const MESSAGE_RESERVATION_CONFIRMATION: &str =
    include_str!("biztalk/RESERVATION_CONFIRMATION_02.txt").trim_ascii();
const MESSAGE_RESERVATION_REMINDER: &str =
    include_str!("biztalk/RESERVATION_REMINDER_02.txt").trim_ascii();
const MESSAGE_RESERVATION_AMEND: &str =
    include_str!("biztalk/RESERVATION_AMEND_01.txt").trim_ascii();

const TEMPLATE_AUDIO_READY: &str = "AUDIO_READY_01";
const TEMPLATE_RESERVATION_CANCEL_CONFIRM: &str = "RESERVATION_CONFIRM_01";
const TEMPLATE_RESERVATION_CANCEL_NO_REFUND: &str = "RESERVATION_CANCEL_NRF_01";
const TEMPLATE_RESERVATION_CANCEL_HALF_REFUND: &str = "RESERVATION_CANCEL_HRF_01";
const TEMPLATE_RESERVATION_CANCEL_FULL_REFUND: &str = "RESERVATION_CANCEL_RF_01";
const TEMPLATE_RESERVATION_CONFIRMATION: &str = "RESERVATION_CONFIRMATION_02";
const TEMPLATE_RESERVATION_REMINDER: &str = "RESERVATION_REMINDER_02";
const TEMPLATE_RESERVATION_AMEND: &str = "RESERVATION_AMEND_01";

pub type BiztalkRecipient = String;
pub type BiztalkSender = super::MessagingSender<BiztalkRecipient>;

#[derive(Debug, Clone)]
pub(super) struct BiztalkClient {
    client: dxe_extern::biztalk::BiztalkClient,
    timezone_config: TimeZoneConfig,
    url_config: UrlConfig,
}

impl BiztalkClient {
    pub fn new(
        config: &BiztalkConfig,
        timezone_config: TimeZoneConfig,
        url_config: UrlConfig,
    ) -> Self {
        Self {
            client: dxe_extern::biztalk::BiztalkClient::new(config),
            timezone_config,
            url_config,
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
        booking: Booking,
    ) -> Result<(), Self::Error> {
        let start = self.timezone_config.convert(booking.time_from);
        let end = self.timezone_config.convert(booking.time_to);

        let time_str = format!(
            "{} - {} ({} 시간)",
            start.format("%Y-%m-%d %H:%M"),
            end.format("%Y-%m-%d %H:%M"),
            (end - start).num_hours()
        );

        let message = MESSAGE_RESERVATION_CONFIRMATION
            .replace("#{customer}", booking.customer.name())
            .replace("#{reservation_dt}", &time_str);

        let mut url = self.url_config.base_url.clone();
        url.set_path(&format!("reservation/{}", booking.id));

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
                        url_mobile: url.to_string(),
                        url_pc: Some(url.to_string()),
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

    async fn send_booking_reminder(
        &self,
        recipients: Vec<Self::Recipient>,
        booking: Booking,
    ) -> Result<(), Self::Error> {
        let start = self.timezone_config.convert(booking.time_from);
        let end = self.timezone_config.convert(booking.time_to);

        let relative_time_str = {
            let mut dt = String::new();

            let now = Utc::now();
            let remaining = booking.time_from - now;
            let hours = remaining.num_hours();
            let minutes = remaining.num_minutes() - (hours * 60);

            if hours > 0 {
                dt.push_str(&format!("{hours}시간 "));
            }
            if minutes > 0 {
                dt.push_str(&format!("{minutes}분 "));
            }

            dt
        };

        let time_str = format!(
            "{} - {} ({} 시간)",
            start.format("%Y-%m-%d %H:%M"),
            end.format("%Y-%m-%d %H:%M"),
            (end - start).num_hours()
        );

        let message = MESSAGE_RESERVATION_REMINDER
            .replace("#{customer}", booking.customer.name())
            .replace("#{reservation_relative_time}", relative_time_str.trim())
            .replace("#{reservation_dt}", &time_str);

        let mut url = self.url_config.base_url.clone();
        url.set_path(&format!("reservation/{}", booking.id));

        let mut error = None;

        for recipient in recipients {
            if let Err(e) = self
                .client
                .send_alimtalk(
                    &recipient,
                    TEMPLATE_RESERVATION_REMINDER,
                    message.clone(),
                    Some(vec![AlimTalkButtonAttachment {
                        name: "예약 확인".to_owned(),
                        r#type: Default::default(),
                        url_mobile: url.to_string(),
                        url_pc: Some(url.to_string()),
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

    async fn send_amend_notification(
        &self,
        recipients: Vec<Self::Recipient>,
        booking: Booking,
        new_time_from: DateTime<Utc>,
        new_time_to: DateTime<Utc>,
    ) -> Result<(), Self::Error> {
        let prev_start = self.timezone_config.convert(booking.time_from);
        let prev_end = self.timezone_config.convert(booking.time_to);

        let prev_time_str = format!(
            "{} - {} ({} 시간)",
            prev_start.format("%Y-%m-%d %H:%M"),
            prev_end.format("%Y-%m-%d %H:%M"),
            (prev_end - prev_start).num_hours()
        );

        let new_start = self.timezone_config.convert(new_time_from);
        let new_end = self.timezone_config.convert(new_time_to);

        let new_time_str = format!(
            "{} - {} ({} 시간)",
            new_start.format("%Y-%m-%d %H:%M"),
            new_end.format("%Y-%m-%d %H:%M"),
            (new_end - new_start).num_hours()
        );

        let message = MESSAGE_RESERVATION_AMEND
            .replace("#{customer}", booking.customer.name())
            .replace("#{old_reservation_dt}", &prev_time_str)
            .replace("#{new_reservation_dt}", &new_time_str);

        let mut url = self.url_config.base_url.clone();
        url.set_path(&format!("reservation/{}", booking.id));

        let mut error = None;

        for recipient in recipients {
            if let Err(e) = self
                .client
                .send_alimtalk(
                    &recipient,
                    TEMPLATE_RESERVATION_AMEND,
                    message.clone(),
                    Some(vec![AlimTalkButtonAttachment {
                        name: "예약 확인".to_owned(),
                        r#type: Default::default(),
                        url_mobile: url.to_string(),
                        url_pc: Some(url.to_string()),
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
        booking: Booking,
        refund_rate: i32,
    ) -> Result<(), Self::Error> {
        let start = self.timezone_config.convert(booking.time_from);
        let end = self.timezone_config.convert(booking.time_to);

        let time_str = format!(
            "{} - {} ({} 시간)",
            start.format("%Y-%m-%d %H:%M"),
            end.format("%Y-%m-%d %H:%M"),
            (end - start).num_hours()
        );

        let (template, message) = match refund_rate {
            0 => (
                TEMPLATE_RESERVATION_CANCEL_NO_REFUND,
                MESSAGE_RESERVATION_CANCEL_NRF
                    .replace("#{customer}", booking.customer.name())
                    .replace("#{reservation_dt}", &time_str),
            ),
            50 => (
                TEMPLATE_RESERVATION_CANCEL_HALF_REFUND,
                MESSAGE_RESERVATION_CANCEL_HRF
                    .replace("#{customer}", booking.customer.name())
                    .replace("#{reservation_dt}", &time_str),
            ),
            100 => (
                TEMPLATE_RESERVATION_CANCEL_FULL_REFUND,
                MESSAGE_RESERVATION_CANCEL_RF
                    .replace("#{customer}", booking.customer.name())
                    .replace("#{reservation_dt}", &time_str),
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
        booking: Booking,
        refunded_price: i64,
    ) -> Result<(), Self::Error> {
        let start = self.timezone_config.convert(booking.time_from);
        let end = self.timezone_config.convert(booking.time_to);

        let time_str = format!(
            "{} - {} ({} 시간)",
            start.format("%Y-%m-%d %H:%M"),
            end.format("%Y-%m-%d %H:%M"),
            (end - start).num_hours()
        );

        let message = MESSAGE_RESERVATION_CANCEL_CONFIRM
            .replace("#{customer}", booking.customer.name())
            .replace("#{reservation_dt}", &time_str)
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

    async fn send_audio_recording(
        &self,
        recipients: Vec<Self::Recipient>,
        booking: Booking,
        audio_recording: AudioRecording,
    ) -> Result<(), Self::Error> {
        let start = self.timezone_config.convert(booking.time_from);
        let end = self.timezone_config.convert(booking.time_to);

        let time_str = format!(
            "{} - {} ({} 시간)",
            start.format("%Y-%m-%d %H:%M"),
            end.format("%Y-%m-%d %H:%M"),
            (end - start).num_hours()
        );

        let expires_in = audio_recording
            .expires_in
            .map(|v| v.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or(String::from("-"));

        let message = MESSAGE_AUDIO_READY
            .replace("#{customer}", booking.customer.name())
            .replace("#{reservation_dt}", &time_str)
            .replace("#{expires_dt}", &expires_in);

        let mut url = self.url_config.base_url.clone();
        url.set_path(&format!("booking/{}/recording", booking.id));

        let mut error = None;

        for recipient in recipients {
            if let Err(e) = self
                .client
                .send_alimtalk(
                    &recipient,
                    TEMPLATE_AUDIO_READY,
                    message.clone(),
                    Some(vec![AlimTalkButtonAttachment {
                        name: "음원 다운로드".to_owned(),
                        r#type: Default::default(),
                        url_mobile: url.to_string(),
                        url_pc: Some(url.to_string()),
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
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No template for refund rate: {0}")]
    NoTemplateForRefundRate(i32),
    #[error("Biztalk Error: {0}")]
    Biztalk(#[from] dxe_extern::biztalk::Error),
}
