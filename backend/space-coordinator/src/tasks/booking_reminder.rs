use std::error::Error as StdError;
use std::sync::Arc;

use dxe_s2s_shared::entities::BookingWithUsers;

use crate::client::DxeClient;

#[derive(Debug)]
pub struct BookingReminder {
    client: DxeClient,
}

impl BookingReminder {
    pub fn new(client: DxeClient) -> Arc<Self> {
        Arc::new(Self { client })
    }

    pub async fn send_reminder(&self, booking: &BookingWithUsers) -> Result<(), Box<dyn StdError>> {
        if let Err(e) = self
            .client
            .post::<serde_json::Value, serde_json::Value>(
                &format!("/booking/{}/reminder", booking.booking.id),
                None,
                serde_json::json!({}),
            )
            .await
        {
            log::warn!(
                "Could not send reminder for booking {}: {e}",
                booking.booking.id
            );

            Err(Box::new(e))
        } else {
            Ok(())
        }
    }
}
