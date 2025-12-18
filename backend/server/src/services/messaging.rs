pub mod biztalk;

use chrono::{DateTime, Utc};
use dxe_data::entities::{AudioRecording, Booking, Identity};
use dxe_data::queries::identity::get_group_members;
use dxe_types::IdentityProvider;
use sqlx::SqliteConnection;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::config::{MessagingConfig, TimeZoneConfig, UrlConfig};
use crate::models::Error;
use crate::services::messaging::biztalk::{BiztalkClient, BiztalkSender};

#[derive(Debug)]
pub enum MessagingEvent<R> {
    BookingConfirmation {
        recipients: Vec<R>,
        booking: Booking,
    },
    BookingReminder {
        recipients: Vec<R>,
        booking: Booking,
    },
    AmendNotification {
        recipients: Vec<R>,
        booking: Booking,
        new_time_from: DateTime<Utc>,
        new_time_to: DateTime<Utc>,
    },
    CancelNotification {
        recipients: Vec<R>,
        booking: Booking,
        refund_rate: i32,
    },
    RefundNotification {
        recipient: R,
        booking: Booking,
        refunded_price: i64,
    },
    AudioRecording {
        recipients: Vec<R>,
        booking: Booking,
        audio_recording: AudioRecording,
    },
}

#[derive(Clone, Debug)]
pub struct MessagingSender<R> {
    tx: mpsc::UnboundedSender<MessagingEvent<R>>,
}

impl<R> MessagingSender<R> {
    pub fn send(&self, event: MessagingEvent<R>) {
        let _ = self.tx.send(event);
    }
}

#[async_trait::async_trait]
pub trait MessagingBackend {
    type Recipient;
    type Error: std::error::Error;

    async fn send_booking_confirmation(
        &self,
        recipients: Vec<Self::Recipient>,
        booking: Booking,
    ) -> Result<(), Self::Error>;
    async fn send_booking_reminder(
        &self,
        recipients: Vec<Self::Recipient>,
        booking: Booking,
    ) -> Result<(), Self::Error>;
    async fn send_amend_notification(
        &self,
        recipients: Vec<Self::Recipient>,
        booking: Booking,
        new_time_from: DateTime<Utc>,
        new_time_to: DateTime<Utc>,
    ) -> Result<(), Self::Error>;
    async fn send_cancel_notification(
        &self,
        recipients: Vec<Self::Recipient>,
        booking: Booking,
        refund_rate: i32,
    ) -> Result<(), Self::Error>;
    async fn send_refund_confirmation(
        &self,
        recipient: Self::Recipient,
        booking: Booking,
        refunded_price: i64,
    ) -> Result<(), Self::Error>;
    async fn send_audio_recording(
        &self,
        recipients: Vec<Self::Recipient>,
        booking: Booking,
        audio_recording: AudioRecording,
    ) -> Result<(), Self::Error>;
}

pub fn spawn_messaging_backend<B, R>(
    backend: B,
) -> (tokio::task::JoinHandle<()>, MessagingSender<R>)
where
    B: MessagingBackend<Recipient = R> + Send + 'static,
    R: Send + 'static,
{
    let (tx, mut rx) = mpsc::unbounded_channel();

    let task = tokio::task::spawn(async move {
        while let Some(command) = rx.recv().await {
            match command {
                MessagingEvent::BookingConfirmation {
                    recipients,
                    booking,
                } => {
                    if let Err(e) = backend.send_booking_confirmation(recipients, booking).await {
                        log::warn!("Could not send booking confirmation: {e}");
                    }
                }
                MessagingEvent::BookingReminder {
                    recipients,
                    booking,
                } => {
                    if let Err(e) = backend.send_booking_reminder(recipients, booking).await {
                        log::warn!("Could not send booking confirmation: {e}");
                    }
                }
                MessagingEvent::AmendNotification {
                    recipients,
                    booking,
                    new_time_from,
                    new_time_to,
                } => {
                    if let Err(e) = backend
                        .send_amend_notification(recipients, booking, new_time_from, new_time_to)
                        .await
                    {
                        log::warn!("Could not send booking confirmation: {e}");
                    }
                }
                MessagingEvent::CancelNotification {
                    recipients,
                    booking,
                    refund_rate,
                } => {
                    if let Err(e) = backend
                        .send_cancel_notification(recipients, booking, refund_rate)
                        .await
                    {
                        log::warn!("Could not send cancel notification: {e}");
                    }
                }
                MessagingEvent::RefundNotification {
                    recipient,
                    booking,
                    refunded_price,
                } => {
                    if let Err(e) = backend
                        .send_refund_confirmation(recipient, booking, refunded_price)
                        .await
                    {
                        log::warn!("Could not send refund confirmation: {e}");
                    }
                }
                MessagingEvent::AudioRecording {
                    recipients,
                    booking,
                    audio_recording,
                } => {
                    if let Err(e) = backend
                        .send_audio_recording(recipients, booking, audio_recording)
                        .await
                    {
                        log::warn!("Could not send audio recording: {e}")
                    }
                }
            }
        }
    });

    (task, MessagingSender { tx })
}

pub struct MessagingService {
    biztalk_sender: Option<BiztalkSender>,
}

impl MessagingService {
    pub fn new(
        config: &MessagingConfig,
        timezone_config: TimeZoneConfig,
        url_config: UrlConfig,
    ) -> (Self, Vec<JoinHandle<()>>) {
        let mut tasks = vec![];

        let biztalk_sender = if let Some(biztalk_config) = &config.biztalk {
            let biztalk_client = BiztalkClient::new(biztalk_config, timezone_config, url_config);
            let (biztalk_task, biztalk_sender) = spawn_messaging_backend(biztalk_client);

            tasks.push(biztalk_task);

            Some(biztalk_sender)
        } else {
            None
        };

        (Self { biztalk_sender }, tasks)
    }

    pub async fn send_confirmation(
        &self,
        database: &mut SqliteConnection,
        booking: Booking,
    ) -> Result<(), Error> {
        let recipients = match &booking.customer {
            Identity::Group(g) => get_group_members(&mut *database, &g.id).await?,
            Identity::User(u) => vec![u.clone()],
        };

        if let Some(biztalk_sender) = &self.biztalk_sender {
            let biztalk_recipients: Vec<_> = recipients
                .iter()
                .filter_map(|v| {
                    if v.provider == IdentityProvider::Kakao {
                        Some(v.foreign_id.clone())
                    } else {
                        None
                    }
                })
                .collect();

            biztalk_sender.send(MessagingEvent::BookingConfirmation {
                recipients: biztalk_recipients,
                booking,
            });
        }

        Ok(())
    }

    pub async fn send_reminder(
        &self,
        database: &mut SqliteConnection,
        booking: Booking,
    ) -> Result<(), Error> {
        let recipients = match &booking.customer {
            Identity::Group(g) => get_group_members(&mut *database, &g.id).await?,
            Identity::User(u) => vec![u.clone()],
        };

        if let Some(biztalk_sender) = &self.biztalk_sender {
            let biztalk_recipients: Vec<_> = recipients
                .iter()
                .filter_map(|v| {
                    if v.provider == IdentityProvider::Kakao {
                        Some(v.foreign_id.clone())
                    } else {
                        None
                    }
                })
                .collect();

            biztalk_sender.send(MessagingEvent::BookingReminder {
                recipients: biztalk_recipients,
                booking,
            });
        }

        Ok(())
    }

    pub async fn send_amend_notification(
        &self,
        database: &mut SqliteConnection,
        booking: Booking,
        new_time_from: DateTime<Utc>,
        new_time_to: DateTime<Utc>,
    ) -> Result<(), Error> {
        let recipients = match &booking.customer {
            Identity::Group(g) => get_group_members(&mut *database, &g.id).await?,
            Identity::User(u) => vec![u.clone()],
        };

        if let Some(biztalk_sender) = &self.biztalk_sender {
            let biztalk_recipients: Vec<_> = recipients
                .iter()
                .filter_map(|v| {
                    if v.provider == IdentityProvider::Kakao {
                        Some(v.foreign_id.clone())
                    } else {
                        None
                    }
                })
                .collect();

            biztalk_sender.send(MessagingEvent::AmendNotification {
                recipients: biztalk_recipients,
                booking,
                new_time_from,
                new_time_to,
            });
        }

        Ok(())
    }

    pub async fn send_cancellation(
        &self,
        database: &mut SqliteConnection,
        booking: Booking,
        refund_rate: i32,
    ) -> Result<(), Error> {
        let recipients = match &booking.customer {
            Identity::Group(g) => get_group_members(&mut *database, &g.id).await?,
            Identity::User(u) => vec![u.clone()],
        };

        if let Some(biztalk_sender) = &self.biztalk_sender {
            let biztalk_recipients: Vec<_> = recipients
                .iter()
                .filter_map(|v| {
                    if v.provider == IdentityProvider::Kakao {
                        Some(v.foreign_id.clone())
                    } else {
                        None
                    }
                })
                .collect();

            biztalk_sender.send(MessagingEvent::CancelNotification {
                recipients: biztalk_recipients,
                booking,
                refund_rate,
            });
        }

        Ok(())
    }

    pub fn send_refund_confirmation(&self, booking: Booking, refunded_price: i64) {
        #[allow(clippy::single_match)]
        match booking.holder.provider {
            IdentityProvider::Kakao => {
                if let Some(biztalk_sender) = &self.biztalk_sender {
                    biztalk_sender.send(MessagingEvent::RefundNotification {
                        recipient: booking.holder.foreign_id.clone(),
                        booking,
                        refunded_price,
                    });
                }
            }
            _ => {}
        }
    }

    pub async fn send_audio_recording(
        &self,
        database: &mut SqliteConnection,
        booking: Booking,
        audio_recording: AudioRecording,
    ) -> Result<(), Error> {
        let recipients = match &booking.customer {
            Identity::Group(g) => get_group_members(&mut *database, &g.id).await?,
            Identity::User(u) => vec![u.clone()],
        };

        if let Some(biztalk_sender) = &self.biztalk_sender {
            let biztalk_recipients: Vec<_> = recipients
                .iter()
                .filter_map(|v| {
                    if v.provider == IdentityProvider::Kakao {
                        Some(v.foreign_id.clone())
                    } else {
                        None
                    }
                })
                .collect();

            biztalk_sender.send(MessagingEvent::AudioRecording {
                recipients: biztalk_recipients,
                booking,
                audio_recording,
            });
        }

        Ok(())
    }
}
