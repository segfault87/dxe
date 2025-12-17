use std::sync::Arc;

use dxe_s2s_shared::entities::BookingWithUsers;

use crate::callback::EventStateCallback;
use crate::client::DxeClient;

#[derive(Debug)]
pub struct BookingReminder {
    client: DxeClient,
}

impl BookingReminder {
    pub fn new(client: DxeClient) -> Arc<Self> {
        Arc::new(Self { client })
    }
}

#[async_trait::async_trait]
impl EventStateCallback<BookingWithUsers> for BookingReminder {
    async fn on_event_start(
        self: Arc<Self>,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if buffered {
            if let Err(e) = self
                .client
                .post::<serde_json::Value, serde_json::Value>(
                    &format!("/booking/{}/reminder", event.booking.id),
                    None,
                    serde_json::json!({}),
                )
                .await
            {
                log::warn!(
                    "Could not send reminder for booking {}: {e}",
                    event.booking.id
                );

                Err(Box::new(e))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}
