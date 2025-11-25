use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::{Arc, Mutex};

use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_types::{BookingId, UnitId};
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::callback::EventStateCallback;
use crate::config::z2m;
use crate::services::mqtt::{DeviceName, IncomingMessage, MqttService};
use crate::tasks::presence_monitor::PresenceState;

pub struct PerUnitMqttController {
    unit_id: UnitId,
    presence_state: Arc<Mutex<PresenceState>>,
    mqtt_service: MqttService,

    devices: HashMap<DeviceName, z2m::Device>,
    device_states: Mutex<HashMap<DeviceName, serde_json::Value>>,
    hooks: z2m::Hooks,
    alerts: Vec<z2m::Alert>,

    active_bookings: Mutex<HashSet<BookingId>>,
}

impl PerUnitMqttController {
    pub fn new(
        unit_id: UnitId,
        config: &z2m::Config,
        mqtt_service: MqttService,
        presence_state: Arc<Mutex<PresenceState>>,
    ) -> Self {
        Self {
            presence_state,
            mqtt_service,

            devices: config
                .devices
                .iter()
                .map(|v| (v.name.clone(), v.clone()))
                .collect(),
            device_states: Mutex::new(HashMap::new()),
            hooks: config.hooks.get(&unit_id).cloned().unwrap_or_default(),
            alerts: config.alerts.clone(),

            unit_id,

            active_bookings: Mutex::new(Default::default()),
        }
    }

    pub async fn handle_incoming_message(
        self: Arc<Self>,
        device_name: DeviceName,
        payload: serde_json::Value,
    ) {
        self.device_states
            .lock()
            .unwrap()
            .insert(device_name, payload);
    }

    async fn update(self: Arc<Self>) {
        log::info!("{:#?}", self.device_states.lock().unwrap());
    }

    pub fn task(self) -> (Arc<Self>, JoinHandle<()>, Task) {
        let task_name = format!("mqtt_controller_{}", self.unit_id);

        let mut receiver = self.mqtt_service.receiver();

        let arc_self = Arc::new(self);

        let inner_arc_self = arc_self.clone();
        let handle = tokio::spawn(async move {
            while let Ok((device_name, value)) = receiver.recv().await {
                Arc::clone(&inner_arc_self)
                    .handle_incoming_message(device_name, value)
                    .await;
            }
        });

        (
            arc_self.clone(),
            handle,
            TaskBuilder::new(&task_name, move || {
                let arc_self = arc_self.clone();
                tokio::task::spawn(async move {
                    arc_self.update().await;
                });

                Ok(())
            })
            //.every_minutes(1)
            .every_seconds(10)
            .build(),
        )
    }
}

#[async_trait::async_trait]
impl EventStateCallback<BookingWithUsers> for PerUnitMqttController {
    async fn on_event_created(
        &self,
        event: &BookingWithUsers,
        is_in_progress: bool,
    ) -> Result<(), Box<dyn Error>> {
        if event.booking.unit_id != self.unit_id {
            return Ok(());
        }

        if is_in_progress {
            self.active_bookings
                .lock()
                .unwrap()
                .insert(event.booking.id.clone());
        }

        Ok(())
    }

    async fn on_event_deleted(
        &self,
        event: &BookingWithUsers,
        is_in_progress: bool,
    ) -> Result<(), Box<dyn Error>> {
        if event.booking.unit_id != self.unit_id {
            return Ok(());
        }

        if is_in_progress {
            self.active_bookings
                .lock()
                .unwrap()
                .remove(&event.booking.id);
        }

        Ok(())
    }

    async fn on_event_start(
        &self,
        event: &BookingWithUsers,
        _buffered: bool,
    ) -> Result<(), Box<dyn Error>> {
        if event.booking.unit_id != self.unit_id {
            return Ok(());
        }

        self.active_bookings
            .lock()
            .unwrap()
            .insert(event.booking.id.clone());

        Ok(())
    }

    async fn on_event_end(
        &self,
        event: &BookingWithUsers,
        _buffered: bool,
    ) -> Result<(), Box<dyn Error>> {
        if event.booking.unit_id != self.unit_id {
            return Ok(());
        }

        self.active_bookings
            .lock()
            .unwrap()
            .remove(&event.booking.id);

        Ok(())
    }
}
