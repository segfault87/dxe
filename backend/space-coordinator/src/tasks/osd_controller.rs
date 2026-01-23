pub mod topics;
pub mod types;

use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::sync::Arc;

use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_types::{BookingId, UnitId};
use parking_lot::Mutex;
use rumqttc::Publish;
use serde::Serialize;
use tokio::task::JoinHandle;
use tokio_task_scheduler::{Scheduler, Task, TaskBuilder, TaskStatus};

use crate::callback::{AlertCallback, EventStateCallback};
use crate::client::DxeClient;
use crate::config::osd::{AlertConfig, AlertKind, Config as OsdConfig, MixerConfig};
use crate::services::mqtt::{Error as MqttError, MqttService, MqttTopicPrefix};
use crate::tasks::osd_controller::topics::DoorLockOpenResult;
use crate::tasks::unit_fetcher::UnitsState;
use crate::types::AlertId;

type BookingEntry = (Option<types::Booking>, HashSet<BookingId>);

pub trait OsdTopic: Serialize {
    fn topic_name(&self) -> String;
}

#[derive(Debug)]
pub struct OsdController {
    client: DxeClient,
    scheduler: Scheduler,
    mqtt_service: MqttService,

    topic_prefix: MqttTopicPrefix,
    alerts: Vec<AlertConfig>,
    mixer_configs: HashMap<UnitId, MixerConfig>,
    doorbell_alert_id: Option<AlertId>,

    units: UnitsState,
    current_bookings: Mutex<HashMap<UnitId, BookingEntry>>,
    sign_off_tasks: Mutex<Vec<String>>,
}

impl OsdController {
    pub fn new(
        client: DxeClient,
        mqtt_service: MqttService,
        units: UnitsState,
        scheduler: Scheduler,
        config: &OsdConfig,
    ) -> Self {
        Self {
            client,
            scheduler,
            mqtt_service,

            topic_prefix: config.topic_prefix.clone(),
            alerts: config.alerts.clone(),
            mixer_configs: config.mixers.clone(),
            doorbell_alert_id: config.doorbell_alert_id.clone(),

            units,
            current_bookings: Mutex::new(HashMap::new()),
            sign_off_tasks: Mutex::new(Vec::new()),
        }
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

    async fn push_current_states(&self) {
        let bookings_per_units = self.current_bookings.lock().clone();
        for (unit_id, (booking, ids)) in bookings_per_units {
            let _ = self
                .publish(&topics::SetScreenState {
                    unit_id: unit_id.clone(),
                    is_active: !ids.is_empty(),
                })
                .await;

            let _ = self
                .publish(&topics::CurrentSession { unit_id, booking })
                .await;
        }
    }

    async fn update(self: Arc<Self>) {
        self.push_current_states().await;

        let mut tasks_to_retain = vec![];
        let sign_off_tasks = self.sign_off_tasks.lock().clone();
        for task in sign_off_tasks {
            if matches!(
                self.scheduler.get_task_status(&task).await,
                Ok(TaskStatus::Completed)
            ) {
                let _ = self.scheduler.remove(&task).await;
            } else {
                tasks_to_retain.push(task);
            }
        }
        *self.sign_off_tasks.lock() = tasks_to_retain;
    }

    pub async fn task(self) -> Result<(Arc<Self>, JoinHandle<()>, Task), Error> {
        self.mqtt_service
            .subscribe(&self.topic_prefix.topic("doorlock/set"))
            .await?;
        let mut subscriber = self.mqtt_service.receiver(self.topic_prefix.clone());

        let arc_self = Arc::new(self);

        let arc_self_inner = arc_self.clone();
        let message_handler_task = tokio::task::spawn(async move {
            while let Ok(message) = subscriber.recv().await {
                arc_self_inner.clone().handle_message(message).await;
            }
        });

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
impl AlertCallback for OsdController {
    async fn on_alert(&self, alert_id: AlertId, started: bool) -> Result<(), Box<dyn StdError>> {
        if let Some(doorbell_alert_id) = &self.doorbell_alert_id
            && doorbell_alert_id == &alert_id
        {
            if let Err(e) = self
                .publish(&topics::DoorbellRequest { unit_id: None })
                .await
            {
                log::warn!("Could not publish doorbell alert: {e}");
            }
            return Ok(());
        }

        for (alert, unit_id) in self.alerts.iter().filter_map(|v| {
            if let AlertKind::Alert { alert_id: id } = &v.kind
                && &alert_id == id
            {
                Some((v.data.clone(), v.unit_id.clone()))
            } else {
                None
            }
        }) {
            for available_unit_id in self.units.get() {
                let alert = if started { Some(alert.clone()) } else { None };

                if unit_id.as_ref().is_none_or(|v| v == &available_unit_id)
                    && let Err(e) = self
                        .publish(&topics::Alert {
                            unit_id: available_unit_id.clone(),
                            alert,
                        })
                        .await
                {
                    log::warn!("Could not publish alert {alert_id} on {available_unit_id}: {e}");
                }
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl EventStateCallback<BookingWithUsers> for OsdController {
    async fn on_event_start(
        self: Arc<Self>,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if buffered {
            if let Err(e) = self
                .publish(&topics::SetScreenState {
                    unit_id: event.booking.unit_id.clone(),
                    is_active: true,
                })
                .await
            {
                log::warn!("Could not send SetScreenState to OSD: {e}");
            }

            if let Some(on_sign_in) = self.alerts.iter().find(|v| {
                matches!(v.kind, AlertKind::OnSignOn)
                    && v.unit_id
                        .as_ref()
                        .is_none_or(|unit_id| unit_id == &event.booking.unit_id)
            }) && self
                .current_bookings
                .lock()
                .entry(event.booking.unit_id.clone())
                .or_default()
                .1
                .is_empty()
                && let Err(e) = self
                    .publish(&topics::Alert {
                        unit_id: event.booking.unit_id.clone(),
                        alert: Some(on_sign_in.data.clone()),
                    })
                    .await
            {
                log::warn!("Could not send sign in alert to OSD: {e}");
            }

            self.current_bookings
                .lock()
                .get_mut(&event.booking.unit_id)
                .unwrap()
                .1
                .insert(event.booking.id);
        } else {
            if let Some(mixer_config) = self.mixer_configs.get(&event.booking.unit_id) {
                let set_mixer_state = topics::SetMixerStates {
                    unit_id: event.booking.unit_id.clone(),
                    channels: mixer_config.channels.clone(),
                    globals: Some(mixer_config.globals.clone()),
                    overwrite: true,
                };
                let delay = mixer_config.reset_after;
                let cloned_self = self.clone();
                tokio::task::spawn(async move {
                    tokio::time::sleep(delay.to_std().unwrap()).await;
                    if let Err(e) = cloned_self.publish(&set_mixer_state).await {
                        log::warn!("Could not send mixer states to OSD: {e}");
                    }
                });
            }

            let booking = types::Booking {
                booking_id: event.booking.id,
                customer_name: event.booking.customer_name.clone(),
                time_from: event.booking.date_start.to_utc(),
                time_to: event.booking.date_end.to_utc(),
            };

            if let Some(on_sign_in) = self.alerts.iter().find(|v| {
                matches!(v.kind, AlertKind::OnSignOn)
                    && v.unit_id
                        .as_ref()
                        .is_none_or(|unit_id| unit_id == &event.booking.unit_id)
            }) && self
                .current_bookings
                .lock()
                .entry(event.booking.unit_id.clone())
                .or_default()
                .1
                .len()
                > 1
                && let Err(e) = self
                    .publish(&topics::Alert {
                        unit_id: event.booking.unit_id.clone(),
                        alert: Some(on_sign_in.data.clone()),
                    })
                    .await
            {
                log::warn!("Could not send sign in alert to OSD: {e}");
            }

            if let Err(e) = self
                .publish(&topics::CurrentSession {
                    unit_id: event.booking.unit_id.clone(),
                    booking: Some(booking.clone()),
                })
                .await
            {
                log::warn!("Could not send current session to OSD: {e}");
            }

            self.current_bookings
                .lock()
                .entry(event.booking.unit_id.clone())
                .or_default()
                .0 = Some(booking);

            if let Some((on_sign_off, before)) = self.alerts.iter().find_map(|v| {
                if let AlertKind::OnSignOff { before } = v.kind
                    && v.unit_id
                        .as_ref()
                        .is_none_or(|unit_id| unit_id == &event.booking.unit_id)
                {
                    Some((v, before))
                } else {
                    None
                }
            }) {
                let arc_self = self.clone();
                let unit_id = event.booking.unit_id.clone();
                let time = event.booking.date_end.to_utc() - before;
                let task_name = format!("osd_sign_off_notification_{unit_id}");
                let on_sign_off = on_sign_off.data.clone();
                let task = move || {
                    let arc_self = self.clone();
                    let unit_id = unit_id.clone();
                    let on_sign_off = on_sign_off.clone();
                    tokio::task::spawn(async move {
                        if let Err(e) = arc_self
                            .publish(&topics::Alert {
                                unit_id,
                                alert: Some(on_sign_off),
                            })
                            .await
                        {
                            log::warn!("Could not send sign off alert to OSD: {e}");
                        }
                    });
                    Ok(())
                };
                let task = TaskBuilder::new(&task_name, task)
                    .daily()
                    .at(time.format("%H:%M:%S").to_string().as_str())
                    .unwrap()
                    .build();

                log::info!(
                    "Scheduling sign off notification task at {}",
                    time.format("%H:%M:%S")
                );

                arc_self.sign_off_tasks.lock().push(task.id().to_owned());
                let _ = arc_self.scheduler.add_task(task).await;
            }
        }

        Ok(())
    }

    async fn on_event_end(
        self: Arc<Self>,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if !buffered {
            let remaining_count = self
                .current_bookings
                .lock()
                .get(&event.booking.unit_id)
                .unwrap()
                .1
                .len();

            if remaining_count == 1 {
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
                self.current_bookings
                    .lock()
                    .get_mut(&event.booking.unit_id)
                    .unwrap()
                    .0 = None;
            }
        } else {
            self.current_bookings
                .lock()
                .get_mut(&event.booking.unit_id)
                .unwrap()
                .1
                .remove(&event.booking.id);
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
