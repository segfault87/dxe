pub mod topics;
pub mod types;

use std::collections::HashSet;
use std::error::Error as StdError;
use std::sync::Arc;

use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_types::BookingId;
use parking_lot::Mutex;
use rumqttc::Publish;
use serde::Serialize;
use tokio::task::JoinHandle;

use crate::callback::EventStateCallback;
use crate::client::DxeClient;
use crate::config::OsdConfig;
use crate::services::mqtt::{Error as MqttError, MqttService, MqttTopicPrefix};
use crate::tasks::osd_controller::topics::DoorLockOpenResult;

pub trait OsdTopic: Serialize {
    fn topic_name(&self) -> String;
}

#[derive(Debug)]
pub struct OsdController {
    client: DxeClient,
    mqtt_service: MqttService,
    topic_prefix: MqttTopicPrefix,

    active_bookings: Mutex<HashSet<BookingId>>,
}

impl OsdController {
    pub async fn new(
        client: DxeClient,
        mqtt_service: MqttService,
        config: &OsdConfig,
    ) -> Result<(Arc<Self>, JoinHandle<()>), Error> {
        mqtt_service
            .subscribe(&config.topic_prefix.topic("doorlock/set"))
            .await?;
        let mut subscriber = mqtt_service.receiver(config.topic_prefix.clone());

        let controller = Arc::new(Self {
            client,
            mqtt_service,
            topic_prefix: config.topic_prefix.clone(),

            active_bookings: Mutex::new(HashSet::new()),
        });

        let controller_inner = controller.clone();
        let task = tokio::task::spawn(async move {
            while let Ok(message) = subscriber.recv().await {
                controller_inner.clone().handle_message(message).await;
            }
        });

        Ok((controller, task))
    }

    async fn handle_message(self: Arc<Self>, message: Publish) {
        if message.topic == self.topic_prefix.topic("doorlock/set") {
            let result = match self
                .client
                .post::<serde_json::Value, serde_json::Value>(
                    "/doorlock",
                    None,
                    serde_json::Value::Null,
                )
                .await
            {
                Ok(_) => DoorLockOpenResult {
                    success: true,
                    error: None,
                },
                Err(e) => DoorLockOpenResult {
                    success: false,
                    error: Some(e.to_string()),
                },
            };

            if let Err(e) = self.publish(&result).await {
                log::warn!("Could not publish door lock open result: {e}");
            }
        }
    }

    pub async fn publish<T: OsdTopic>(&self, data: &T) -> Result<(), Error> {
        let topic = self.topic_prefix.topic(&data.topic_name());
        let payload = serde_json::to_vec(data)?;

        self.mqtt_service
            .publish(topic.as_str(), payload.as_slice())
            .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl EventStateCallback<BookingWithUsers> for OsdController {
    async fn on_event_start(
        &self,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if buffered {
            self.active_bookings.lock().insert(event.booking.id);
            if let Err(e) = self
                .publish(&topics::SetScreenState {
                    unit_id: event.booking.unit_id.clone(),
                    is_active: true,
                })
                .await
            {
                log::warn!("Could not send SetScreenState to OSD: {e}");
            }
            if let Err(e) = self
                .publish(&topics::StartSession {
                    unit_id: event.booking.unit_id.clone(),
                    booking: types::Booking {
                        booking_id: event.booking.id,
                        customer_name: event.booking.customer_name.clone(),
                        time_from: event.booking.date_start.to_utc(),
                        time_to: event.booking.date_end.to_utc(),
                    },
                })
                .await
            {
                log::warn!("Could not send StartSession to OSD: {e}");
            }
        }

        Ok(())
    }

    async fn on_event_end(
        &self,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if !buffered {
            self.active_bookings.lock().remove(&event.booking.id);
            if let Err(e) = self
                .publish(&topics::StopSession {
                    unit_id: event.booking.unit_id.clone(),
                    booking_id: event.booking.id,
                })
                .await
            {
                log::warn!("Could not send StopSession to OSD: {e}");
            }

            if self.active_bookings.lock().is_empty()
                && let Err(e) = self
                    .publish(&topics::SetScreenState {
                        unit_id: event.booking.unit_id.clone(),
                        is_active: false,
                    })
                    .await
            {
                log::warn!("Could not send SetScreenState to OSD: {e}");
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("MQTT error: {0}")]
    Mqtt(#[from] MqttError),
    #[error("JSON serialization failed: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
