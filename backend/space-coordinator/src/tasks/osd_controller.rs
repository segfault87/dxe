pub mod topics;
pub mod types;

use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::sync::Arc;

use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_s2s_shared::handlers::{GetMixerConfigResponse, UpdateMixerConfigRequest};
use dxe_types::entities::MixerPresets;
use dxe_types::{BookingId, IdentityId, UnitId};
use futures::StreamExt;
use parking_lot::Mutex;
use rumqttc::Publish;
use serde::Serialize;
use tokio::task::JoinHandle;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::callback::LifecycleEventCallback;
use crate::client::DxeClient;
use crate::config::osd::Config as OsdConfig;
use crate::events::{Event, EventSender};
use crate::services::mqtt::{Error as MqttError, MqttService, MqttTopicPrefix};
use crate::tasks::osd_controller::topics::{Alert, DoorLockOpenResult};
use crate::tasks::osd_controller::types::AlertData;
use crate::types::EventId;

pub trait OsdTopic: Serialize {
    fn topic_name(&self) -> String;
}

#[derive(Debug)]
pub struct OsdController {
    client: DxeClient,
    mqtt_service: MqttService,

    topic_prefix: MqttTopicPrefix,
    alerts: HashMap<EventId, Vec<AlertData>>,
    mixer_configs: HashMap<UnitId, MixerPresets>,
    doorbell_event_id: Option<EventId>,

    event_receiver: Mutex<Option<JoinHandle<()>>>,
    active_sessions: Mutex<HashMap<UnitId, HashSet<BookingId>>>,
}

impl OsdController {
    pub fn new(client: DxeClient, mqtt_service: MqttService, config: &OsdConfig) -> Self {
        let mut alerts: HashMap<EventId, Vec<AlertData>> = HashMap::new();

        for alert in config.alerts.iter() {
            for event_id in alert.event_ids.iter() {
                alerts
                    .entry(event_id.clone())
                    .or_default()
                    .push(alert.data.clone());
            }
        }

        Self {
            client,
            mqtt_service,

            topic_prefix: config.topic_prefix.clone(),
            alerts,
            mixer_configs: config.mixers.clone(),
            doorbell_event_id: config.doorbell_event_id.clone(),

            event_receiver: Mutex::new(None),
            active_sessions: Mutex::new(HashMap::new()),
        }
    }

    async fn handle_doorlock(self: Arc<Self>) {
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

    async fn handle_mixer_preferences(
        self: Arc<Self>,
        unit_id: UnitId,
        payload: types::UpdateMixerConfig,
    ) {
        match self
            .client
            .post::<_, serde_json::Value>(
                "/mixer",
                None,
                UpdateMixerConfigRequest {
                    identity_id: payload.customer_id,
                    unit_id: unit_id.clone(),
                    prefs: payload.prefs.into(),
                },
            )
            .await
        {
            Ok(_) => log::info!(
                "Mixer data updated for {} on {unit_id}",
                payload.customer_id
            ),
            Err(e) => log::warn!("Could not update mixer preferences: {e}"),
        }
    }

    async fn handle_message(self: Arc<Self>, message: Publish) {
        if message.topic == self.topic_prefix.topic("doorlock/set") {
            self.clone().handle_doorlock().await;
        } else if let topic = self.topic_prefix.topic("mixer_preferences/")
            && message.topic.starts_with(&topic)
            && message.topic.ends_with("/set")
        {
            let Some(unit_id) = message
                .topic
                .strip_prefix(&topic)
                .and_then(|v| v.strip_suffix("/set"))
            else {
                log::warn!(
                    "Invalid unit id for mixer_preferences/+/set: {}",
                    message.topic
                );
                return;
            };
            let unit_id = UnitId::from(unit_id.to_owned());

            match serde_json::from_slice::<types::UpdateMixerConfig>(&message.payload) {
                Ok(payload) => {
                    self.clone()
                        .handle_mixer_preferences(unit_id, payload)
                        .await
                }
                Err(e) => log::warn!("Could not deserialize mixer preferences: {e}"),
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

    async fn push_current_states(&self) {
        let bookings_per_units = self.active_sessions.lock().clone();
        for (unit_id, booking_ids) in bookings_per_units {
            let _ = self
                .publish(&topics::SetScreenState {
                    unit_id: unit_id.clone(),
                    is_active: !booking_ids.is_empty(),
                })
                .await;
        }
    }

    async fn send_mixer_states(self: Arc<Self>, unit_id: UnitId, identity_id: IdentityId) {
        let prefs = match self
            .client
            .get::<GetMixerConfigResponse>(
                "/mixer",
                Some(&format!("unit_id={unit_id}&identity_id={identity_id}")),
            )
            .await
        {
            Ok(v) => v.prefs,
            Err(e) => {
                log::warn!("Could not get mixer config for {identity_id}: {e}");
                None
            }
        };

        let prefs =
            prefs
                .map(types::MixerPreferences::from)
                .unwrap_or_else(|| types::MixerPreferences {
                    default: self
                        .mixer_configs
                        .get(&unit_id)
                        .cloned()
                        .unwrap_or_else(Default::default)
                        .into(),
                    scenes: Default::default(),
                });

        let topic = topics::SetMixerPreferences {
            unit_id: unit_id.clone(),
            prefs,
        };

        if let Err(e) = self.publish(&topic).await {
            log::warn!("Could not send mixer states to OSD: {e}");
        }
    }

    async fn update(self: Arc<Self>) {
        self.push_current_states().await;
    }

    pub async fn start(
        self,
        event_sender: EventSender,
    ) -> Result<(Arc<Self>, JoinHandle<()>, Task), Error> {
        self.mqtt_service
            .subscribe(&self.topic_prefix.topic("doorlock/set"))
            .await?;
        self.mqtt_service
            .subscribe(&self.topic_prefix.topic("mixer_preferences/+/set"))
            .await?;
        let mut subscriber = self.mqtt_service.receiver(self.topic_prefix.clone());

        let arc_self = Arc::new(self);

        let arc_self_inner = arc_self.clone();
        let message_handler_task = tokio::task::spawn(async move {
            while let Ok(message) = subscriber.recv().await {
                arc_self_inner.clone().handle_message(message).await;
            }
        });

        let mut keys = arc_self.alerts.keys().cloned().collect::<Vec<_>>();
        if let Some(doorbell_request) = &arc_self.doorbell_event_id {
            keys.push(doorbell_request.clone());
        }
        let mut receiver = event_sender.subscribe(keys.into_iter());
        let arc_self_inner = arc_self.clone();
        let event_receiver_task = tokio::task::spawn(async move {
            while let Some(item) = receiver.next().await {
                if let Ok((event_id, event)) = item {
                    if let Some(doorbell_event_id) = &arc_self_inner.doorbell_event_id
                        && &event_id == doorbell_event_id
                    {
                        if let Err(e) = arc_self_inner
                            .publish(&topics::DoorbellRequest { unit_id: None })
                            .await
                        {
                            log::error!("Could not publish doorbell request: {e}");
                        }
                    } else if let Some(alerts) = arc_self_inner.clone().alerts.get(&event_id) {
                        let unit_ids = match event {
                            Event::Alert { unit_ids, .. } => unit_ids,
                            Event::Booking { booking } => {
                                Some(HashSet::from([booking.booking.unit_id.clone()]))
                            }
                            _ => None,
                        }
                        .unwrap_or_else(|| {
                            arc_self_inner
                                .clone()
                                .active_sessions
                                .lock()
                                .keys()
                                .cloned()
                                .collect()
                        });
                        for unit_id in unit_ids {
                            for alert in alerts.iter() {
                                if let Err(e) = arc_self_inner
                                    .publish(&Alert {
                                        unit_id: unit_id.clone(),
                                        alert: Some(alert.clone()),
                                    })
                                    .await
                                {
                                    log::error!(
                                        "Could not send notification for event {event_id}: {e}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        });
        *arc_self.clone().event_receiver.lock() = Some(event_receiver_task);

        let arc_self_inner = arc_self.clone();
        let task = TaskBuilder::new("osd_controller", move || {
            let arc_self_inner = arc_self_inner.clone();
            tokio::task::spawn(async move {
                arc_self_inner.update().await;
            });

            Ok(())
        })
        .every_minutes(1)
        .build();

        Ok((arc_self, message_handler_task, task))
    }
}

#[async_trait::async_trait]
impl LifecycleEventCallback<BookingWithUsers> for OsdController {
    async fn on_start(self: Arc<Self>, event: &BookingWithUsers) -> Result<(), Box<dyn StdError>> {
        if let Err(e) = self
            .publish(&topics::SetScreenState {
                unit_id: event.booking.unit_id.clone(),
                is_active: true,
            })
            .await
        {
            log::warn!("Could not send SetScreenState to OSD: {e}");
        }

        self.clone()
            .send_mixer_states(event.booking.unit_id.clone(), event.booking.customer_id)
            .await;

        let booking = types::Booking {
            booking_id: event.booking.id,
            customer_id: event.booking.customer_id,
            customer_name: event.booking.customer_name.clone(),
            time_from: event.booking.date_start.to_utc(),
            time_to: event.booking.date_end.to_utc(),
        };

        self.clone()
            .send_mixer_states(event.booking.unit_id.clone(), event.booking.customer_id)
            .await;

        if let Err(e) = self
            .publish(&topics::CurrentSession {
                unit_id: event.booking.unit_id.clone(),
                booking: Some(booking.clone()),
            })
            .await
        {
            log::warn!("Could not send current session to OSD: {e}");
        }

        self.clone()
            .active_sessions
            .lock()
            .entry(event.booking.unit_id.clone())
            .or_default()
            .insert(event.booking.id);

        Ok(())
    }

    async fn on_end(self: Arc<Self>, event: &BookingWithUsers) -> Result<(), Box<dyn StdError>> {
        let count = {
            let active_sessions = self.active_sessions.lock();
            active_sessions
                .get(&event.booking.unit_id)
                .map(|v| v.len())
                .unwrap_or_default()
        };

        if count == 0 {
            let _ = self
                .publish(&topics::Alert {
                    unit_id: event.booking.unit_id.clone(),
                    alert: None,
                })
                .await;
            let _ = self
                .publish(&topics::CurrentSession {
                    unit_id: event.booking.unit_id.clone(),
                    booking: None,
                })
                .await;
            let _ = self
                .publish(&topics::SetScreenState {
                    unit_id: event.booking.unit_id.clone(),
                    is_active: false,
                })
                .await;
            let _ = self
                .publish(&topics::ParkingStates {
                    unit_id: event.booking.unit_id.clone(),
                    states: vec![],
                })
                .await;
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
