pub mod biztalk;

use dxe_types::BookingId;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum MessagingEvent<R> {
    BookingConfirmation {
        recipients: Vec<R>,
        booking_id: BookingId,
        customer_name: String,
        reservation_time: String,
    },
    CancelNotification {
        recipients: Vec<R>,
        booking_id: BookingId,
        customer_name: String,
        reservation_time: String,
        refund_rate: i32,
    },
    RefundNotification {
        recipient: R,
        booking_id: BookingId,
        customer_name: String,
        reservation_time: String,
        refunded_price: i64,
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
        booking_id: &BookingId,
        customer_name: &str,
        reservation_time: &str,
    ) -> Result<(), Self::Error>;
    async fn send_cancel_notification(
        &self,
        recipients: Vec<Self::Recipient>,
        booking_id: &BookingId,
        customer_name: &str,
        reservation_time: &str,
        refund_rate: i32,
    ) -> Result<(), Self::Error>;
    async fn send_refund_confirmation(
        &self,
        recipient: Self::Recipient,
        booking_id: &BookingId,
        customer_name: &str,
        reservation_time: &str,
        refunded_price: i64,
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
                    booking_id,
                    customer_name,
                    reservation_time,
                } => {
                    if let Err(e) = backend
                        .send_booking_confirmation(
                            recipients,
                            &booking_id,
                            customer_name.as_str(),
                            reservation_time.as_str(),
                        )
                        .await
                    {
                        log::warn!("Could not send booking confirmation: {e}");
                    }
                }
                MessagingEvent::CancelNotification {
                    recipients,
                    booking_id,
                    customer_name,
                    reservation_time,
                    refund_rate,
                } => {
                    if let Err(e) = backend
                        .send_cancel_notification(
                            recipients,
                            &booking_id,
                            customer_name.as_str(),
                            reservation_time.as_str(),
                            refund_rate,
                        )
                        .await
                    {
                        log::warn!("Could not send cancel notification: {e}");
                    }
                }
                MessagingEvent::RefundNotification {
                    recipient,
                    booking_id,
                    customer_name,
                    reservation_time,
                    refunded_price,
                } => {
                    if let Err(e) = backend
                        .send_refund_confirmation(
                            recipient,
                            &booking_id,
                            customer_name.as_str(),
                            reservation_time.as_str(),
                            refunded_price,
                        )
                        .await
                    {
                        log::warn!("Could not send refund confirmation: {e}");
                    }
                }
            }
        }
    });

    (task, MessagingSender { tx })
}
