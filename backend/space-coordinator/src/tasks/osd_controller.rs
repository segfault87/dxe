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

use crate::callback::EventStateCallback;
use crate::client::DxeClient;
use crate::config::{OsdAlertConfig, OsdConfig};
use crate::services::mqtt::{Error as MqttError, MqttService, MqttTopicPrefix};
use crate::tasks::osd_controller::topics::DoorLockOpenResult;

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
    alerts: OsdAlertConfig,

    current_bookings: Mutex<HashMap<UnitId, BookingEntry>>,
    sign_off_tasks: Mutex<Vec<String>>,
}

impl OsdController {
    pub fn new(
        client: DxeClient,
        mqtt_service: MqttService,
        scheduler: Scheduler,
        config: &OsdConfig,
    ) -> Self {
        Self {
            client,
            scheduler,
            mqtt_service,

            topic_prefix: config.topic_prefix.clone(),
            alerts: config.alerts.clone(),

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

            if let Some(on_sign_in) = &self.alerts.on_sign_in
                && self
                    .current_bookings
                    .lock()
                    .entry(event.booking.unit_id.clone())
                    .or_default()
                    .1
                    .is_empty()
                && let Err(e) = self
                    .publish(&topics::Alert {
                        unit_id: event.booking.unit_id.clone(),
                        alert: Some(on_sign_in.clone()),
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
            let booking = types::Booking {
                booking_id: event.booking.id,
                customer_name: event.booking.customer_name.clone(),
                time_from: event.booking.date_start.to_utc(),
                time_to: event.booking.date_end.to_utc(),
            };

            if let Some(on_sign_in) = &self.alerts.on_sign_in
                && self
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
                        alert: Some(on_sign_in.clone()),
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

            if let Some(on_sign_off) = self.alerts.on_sign_off.clone() {
                let arc_self = self.clone();
                let unit_id = event.booking.unit_id.clone();
                let time = event.booking.date_end.to_utc() - self.alerts.sign_off_duration();
                let task_name = format!("osd_sign_off_notification_{unit_id}");
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
