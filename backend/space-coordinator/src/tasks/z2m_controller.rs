use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::sync::Arc;

use chrono::TimeDelta;
use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_types::{BookingId, UnitId};
use parking_lot::Mutex;
use rumqttc::Publish;
use serde_json::json;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::callback::{EventStateCallback, PresenceCallback};
use crate::config::z2m::{self, SwitchPolicy};
use crate::services::mqtt::{Error as MqttError, MqttService, MqttTopicPrefix};
use crate::tables::{QualifiedPath, SingleTable, TablePublisher};
use crate::tasks::presence_monitor::PresenceState;
use crate::types::{DeviceId, DeviceRef, DeviceType, PublishKey, PublishedValues, Z2mDeviceId};
use crate::utils::boolean::Error as BooleanError;

const MQTT_TOPIC_PREFIX_Z2M: MqttTopicPrefix = MqttTopicPrefix::new_const("zigbee2mqtt");

#[derive(Debug)]
pub enum Z2mPublishTopic {
    Get,
    Set,
}

impl Z2mDeviceId {
    pub fn topic_name(&self, command: Option<Z2mPublishTopic>) -> String {
        format!(
            "{}/{}{}",
            MQTT_TOPIC_PREFIX_Z2M,
            self,
            match command {
                None => "",
                Some(Z2mPublishTopic::Get) => "/get",
                Some(Z2mPublishTopic::Set) => "/set",
            }
        )
    }

    pub fn from_topic_name(topic: &str) -> Option<Self> {
        topic
            .strip_prefix("zigbee2mqtt/")
            .map(|v| v.to_owned().into())
    }
}

impl From<DeviceId> for Z2mDeviceId {
    fn from(value: DeviceId) -> Self {
        value.to_string().into()
    }
}

pub struct Z2mPath;

impl QualifiedPath for Z2mPath {
    type TableKey = DeviceId;
    type Path = DeviceRef;

    fn path(table_key: &Self::TableKey) -> Self::Path {
        DeviceRef {
            r#type: DeviceType::Z2m,
            id: table_key.clone(),
        }
    }
}

pub struct Z2mController {
    mqtt_service: MqttService,

    command_timeout: TimeDelta,

    devices: HashMap<Z2mDeviceId, z2m::Device>,
    per_unit_hooks: HashMap<UnitId, z2m::PerUnitHooks>,
    presence_hooks: z2m::PresenceHooks,

    active_bookings: Mutex<Option<HashMap<UnitId, HashSet<BookingId>>>>,
    presence: Mutex<bool>,

    state_keys: HashMap<Z2mDeviceId, Vec<PublishKey>>,
    table: TablePublisher<DeviceId, DeviceRef, Z2mPath>,
}

impl Z2mController {
    pub fn new(
        config: &z2m::Config,
        mqtt_service: MqttService,
        presence_state: Arc<Mutex<PresenceState>>,
    ) -> Self {
        Self {
            mqtt_service,

            command_timeout: config.command_timeout,

            devices: config
                .devices
                .iter()
                .map(|v| (v.id.clone(), v.clone()))
                .collect(),
            per_unit_hooks: config.hooks.units.clone(),
            presence_hooks: config.hooks.presence.clone(),

            active_bookings: Mutex::new(Default::default()),
            presence: Mutex::new(presence_state.lock().is_present),

            state_keys: config
                .devices
                .iter()
                .map(|v| (v.id.clone(), v.state_keys()))
                .collect(),
            table: TablePublisher::new(),
        }
    }

    pub fn publisher(&self) -> TablePublisher<DeviceId, DeviceRef, Z2mPath> {
        self.table.clone()
    }

    pub async fn start(&mut self) {
        let mut device_count = self.devices.len();
        for (id, device) in self.devices.iter() {
            if let Err(e) = self.mqtt_service.subscribe(&id.topic_name(None)).await {
                log::warn!("Cannot subscribe to {id}: {e}");
                continue;
            }

            if !device.skip_sync {
                let values = match self.get_states(id).await {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("Could not get state for device {id}: {e}");
                        continue;
                    }
                };

                log::info!("Got initial state for {id}");

                let device_id = id.clone().into();
                self.table.replace(device_id, values);
            }
            device_count -= 1;
        }

        if device_count == 0 {
            log::info!("Synchronized z2m devices successfully.");
        } else {
            log::warn!("{device_count} devices were not synchronized. Proceeding...");
        }
    }

    async fn get_states(&self, device_id: &Z2mDeviceId) -> Result<SingleTable, Error> {
        let state_keys = self
            .state_keys
            .get(device_id)
            .map(|keys| {
                keys.iter()
                    .map(|key| (key.clone(), json!({})))
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();
        if state_keys.is_empty() {
            return Ok(Default::default());
        }

        let mut receiver = self.mqtt_service.receiver(MQTT_TOPIC_PREFIX_Z2M);

        let (tx, rx) = oneshot::channel();

        let expected_device_id = device_id.clone();
        tokio::task::spawn(async move {
            loop {
                let Ok(publish) = receiver.recv().await else {
                    break;
                };

                if let Some(id) = Z2mDeviceId::from_topic_name(&publish.topic)
                    && expected_device_id == id
                {
                    let payload = match serde_json::from_slice::<SingleTable>(&publish.payload) {
                        Ok(v) => v,
                        Err(e) => {
                            log::warn!("Invalid JSON string from topic {}: {e}", publish.topic);
                            continue;
                        }
                    };

                    let _ = tx.send(payload);
                    break;
                }
            }
        });

        self.mqtt_service
            .publish(
                &device_id.topic_name(Some(Z2mPublishTopic::Get)),
                serde_json::to_string(&state_keys)?.as_bytes(),
            )
            .await
            .map_err(|e| Error::Mqtt(Box::new(e)))?;

        match timeout(self.command_timeout.to_std().unwrap(), rx).await {
            Ok(v) => Ok(v.map_err(|_| Error::Recv)?),
            Err(_) => Err(Error::Timeout),
        }
    }

    pub async fn set_state(
        &self,
        id: Z2mDeviceId,
        states: &[PublishedValues],
    ) -> Result<(), Error> {
        let mut receiver = self.mqtt_service.receiver(MQTT_TOPIC_PREFIX_Z2M);

        let (tx, rx) = oneshot::channel();

        let mut states_to_inspect = states.to_vec();
        if states.is_empty() {
            return Ok(());
        }

        let mqtt_service = self.mqtt_service.clone();

        tokio::task::spawn(async move {
            let mut first = states_to_inspect.first().unwrap();

            let payload = match serde_json::to_string(&first) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Could not serialize state {first:?}: {e}");
                    return;
                }
            };
            if let Err(e) = mqtt_service
                .publish(
                    &id.topic_name(Some(Z2mPublishTopic::Set)),
                    payload.as_bytes(),
                )
                .await
            {
                log::warn!("Could not publish set command: {e}");
                return;
            }

            loop {
                let Ok(publish) = receiver.recv().await else {
                    break;
                };

                if let Some(device_id) = Z2mDeviceId::from_topic_name(&publish.topic)
                    && id == device_id
                {
                    let payload = match serde_json::from_slice::<PublishedValues>(&publish.payload)
                    {
                        Ok(v) => v,
                        Err(e) => {
                            log::warn!("Invalid JSON string from topic {}: {e}", publish.topic);
                            continue;
                        }
                    };

                    for (key, value) in first.iter() {
                        if let Some(payload_value) = payload.get(key)
                            && payload_value != value
                        {
                            continue;
                        }
                    }

                    states_to_inspect.remove(0);

                    first = if let Some(first) = states_to_inspect.first() {
                        let payload = match serde_json::to_string(&first) {
                            Ok(v) => v,
                            Err(e) => {
                                log::warn!("Could not serialize state {first:?}: {e}");
                                return;
                            }
                        };
                        if let Err(e) = mqtt_service
                            .publish(
                                &id.topic_name(Some(Z2mPublishTopic::Set)),
                                payload.as_bytes(),
                            )
                            .await
                        {
                            log::warn!("Could not publish set command: {e}");
                            return;
                        }
                        first
                    } else {
                        let _ = tx.send(());
                        return;
                    };
                }
            }
        });

        match timeout(self.command_timeout.to_std().unwrap(), rx).await {
            Ok(v) => Ok(v.map_err(|_| Error::Recv)?),
            Err(_) => Err(Error::Timeout),
        }
    }

    pub fn handle_publishment(self: Arc<Self>, publish: Publish) {
        if let Some(device_id) = Z2mDeviceId::from_topic_name(&publish.topic)
            && let Some(device) = self.devices.get(&device_id)
        {
            let values = match serde_json::from_slice::<PublishedValues>(&publish.payload) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Failed to serialize payload for {device_id}: {e}");
                    return;
                }
            };

            self.table.update(device.id.clone().into(), values);
        }
    }

    fn get_switch_state(
        &self,
        device_id: Z2mDeviceId,
        switch: &z2m::DeviceClassSwitch,
    ) -> Result<z2m::SwitchState, Error> {
        let Some(state) = self.table.get_values(&device_id.clone().into()) else {
            return Err(Error::StateNotFound(device_id));
        };

        if switch.is_on.test(&state)? {
            Ok(z2m::SwitchState::On)
        } else {
            Ok(z2m::SwitchState::Off)
        }
    }

    fn is_active_bookings(&self) -> Option<bool> {
        let guard = self.active_bookings.lock();

        Some(guard.as_ref()?.values().any(|v| !v.is_empty()))
    }

    fn get_active_units(&self) -> Vec<UnitId> {
        let guard = self.active_bookings.lock();

        guard
            .as_ref()
            .map(|bookings| {
                bookings
                    .iter()
                    .filter_map(|(k, v)| if !v.is_empty() { Some(k.clone()) } else { None })
                    .collect()
            })
            .unwrap_or_default()
    }

    async fn set_switch(
        &self,
        device_id: Z2mDeviceId,
        switch: &z2m::DeviceClassSwitch,
        state: z2m::SwitchState,
    ) -> Result<bool, Error> {
        let current_state = self.get_switch_state(device_id.clone(), switch)?;

        if current_state == state {
            Ok(false)
        } else {
            log::info!("Setting switch {device_id} to {state}");

            self.set_state(
                device_id.clone(),
                match state {
                    z2m::SwitchState::On => &switch.states.on,
                    z2m::SwitchState::Off => &switch.states.off,
                },
            )
            .await?;
            Ok(true)
        }
    }

    async fn sync_switch_state(
        &self,
        device_id: Z2mDeviceId,
        switch: &z2m::DeviceClassSwitch,
    ) -> Result<bool, Error> {
        let current_state = self.get_switch_state(device_id.clone(), switch)?;

        let Some(is_active_bookings) = self.is_active_bookings() else {
            return Err(Error::NotInitialized);
        };

        let is_present = *self.presence.lock();

        let switch_policy = if is_active_bookings {
            switch.booking_policy
        } else {
            switch.presence_policy
        };

        let is_in_use = is_present || is_active_bookings;

        let desired_state = match switch_policy {
            SwitchPolicy::Uncontrolled => return Ok(false),
            SwitchPolicy::StayOn => {
                if is_in_use {
                    z2m::SwitchState::On
                } else {
                    z2m::SwitchState::Off
                }
            }
            SwitchPolicy::Off => z2m::SwitchState::Off,
        };

        if current_state == desired_state {
            Ok(false)
        } else {
            log::info!("Setting switch {device_id} to {desired_state}");

            self.set_state(
                device_id.clone(),
                match desired_state {
                    z2m::SwitchState::On => &switch.states.on,
                    z2m::SwitchState::Off => &switch.states.off,
                },
            )
            .await?;
            Ok(true)
        }
    }

    async fn sync_switch_states(&self) {
        for device in self.devices.values() {
            if let Some(switch) = &device.classes.switch
                && let Err(e) = self.sync_switch_state(device.id.clone(), switch).await
            {
                log::warn!("Failed to sync switch state for {}: {e}", device.id);
            }
        }
    }

    fn aggregate_switch_states(
        states: &mut HashMap<Z2mDeviceId, z2m::SwitchState>,
        existing_states: &HashMap<Z2mDeviceId, z2m::SwitchState>,
    ) {
        for (k, unit_state) in existing_states.iter() {
            if let Some(new_state) = states.get_mut(k) {
                let new_value = match (unit_state, *new_state) {
                    (z2m::SwitchState::Off, z2m::SwitchState::On) => z2m::SwitchState::On,
                    (z2m::SwitchState::On, z2m::SwitchState::Off) => z2m::SwitchState::On,
                    (_, new_state) => new_state,
                };
                *new_state = new_value;
            }
        }
    }

    async fn trigger_presence_switch_event(&self, enter: bool) {
        let active_units = self.get_active_units();

        let mut switches = if enter {
            &self.presence_hooks.on_enter.switches
        } else {
            &self.presence_hooks.on_leave.switches
        }
        .clone();

        for unit in active_units {
            let Some(hooks) = self.per_unit_hooks.get(&unit) else {
                continue;
            };

            let per_unit_switches = &hooks.on_booking_start.switches;
            Self::aggregate_switch_states(&mut switches, per_unit_switches);
        }

        for (device_id, state) in switches {
            let Some(switch) = self
                .devices
                .get(&device_id)
                .and_then(|v| v.classes.switch.as_ref())
            else {
                log::warn!("Device {device_id} is not a switch.");
                continue;
            };

            if let Err(e) = self.set_switch(device_id.clone(), switch, state).await {
                log::warn!("Could not change switch state of {device_id} to {state}: {e}");
            }
        }
    }

    async fn trigger_unit_event(&self, unit_id: &UnitId, start: bool) {
        let Some(hook) = self.per_unit_hooks.get(unit_id) else {
            return;
        };

        let presence = *self.presence.lock();

        let active_booking_count = {
            let guard = self.active_bookings.lock();
            let Some(active_bookings) = &*guard else {
                return;
            };

            active_bookings
                .get(unit_id)
                .map(|v| v.len())
                .unwrap_or_default()
        };

        if (start && active_booking_count > 1) || (!start && active_booking_count != 0) {
            return;
        }

        let mut switches = if start {
            &hook.on_booking_start.switches
        } else {
            &hook.on_booking_end.switches
        }
        .clone();

        if presence {
            let presence_switches = &self.presence_hooks.on_enter.switches;
            Self::aggregate_switch_states(&mut switches, presence_switches);
        }

        for (device_id, state) in switches {
            let Some(switch) = self
                .devices
                .get(&device_id)
                .and_then(|v| v.classes.switch.as_ref())
            else {
                log::warn!("Device {device_id} is not a switch.");
                continue;
            };

            if let Err(e) = self.set_switch(device_id.clone(), switch, state).await {
                log::warn!("Could not change switch state of {device_id} to {state}: {e}");
            }
        }
    }

    async fn update(self: Arc<Self>) {
        self.sync_switch_states().await;
    }

    pub fn task(self) -> (Arc<Self>, JoinHandle<()>, Task) {
        let task_name = "mqtt_controller".to_string();

        let mut receiver = self.mqtt_service.receiver(MQTT_TOPIC_PREFIX_Z2M);

        let arc_self = Arc::new(self);

        let inner_arc_self = arc_self.clone();
        let handle = tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                Arc::clone(&inner_arc_self).handle_publishment(message);
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
            .every_minutes(1)
            .build(),
        )
    }
}

#[async_trait::async_trait]
impl EventStateCallback<BookingWithUsers> for Z2mController {
    fn on_initialized(&self) {
        *self.active_bookings.lock() = Some(Default::default());
    }

    async fn on_event_start(
        self: Arc<Self>,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if buffered {
            {
                let mut guard = self.active_bookings.lock();
                if let Some(active_bookings) = guard.as_mut() {
                    active_bookings
                        .entry(event.booking.unit_id.clone())
                        .or_default()
                        .insert(event.booking.id);
                }
            }

            self.trigger_unit_event(&event.booking.unit_id, true).await;
        }

        Ok(())
    }

    async fn on_event_end(
        self: Arc<Self>,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if buffered {
            {
                let mut guard = self.active_bookings.lock();
                if let Some(active_bookings) = guard.as_mut() {
                    active_bookings
                        .entry(event.booking.unit_id.clone())
                        .or_default()
                        .remove(&event.booking.id);
                }
            }

            self.trigger_unit_event(&event.booking.unit_id, false).await;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl PresenceCallback for Z2mController {
    async fn on_enter(&self) -> Result<(), Box<dyn StdError>> {
        let changed = {
            let mut presence = self.presence.lock();
            if !*presence {
                *presence = true;
                true
            } else {
                false
            }
        };

        if changed {
            self.trigger_presence_switch_event(true).await;
        }

        Ok(())
    }

    async fn on_leave(&self) -> Result<(), Box<dyn StdError>> {
        let changed = {
            let mut presence = self.presence.lock();
            if *presence {
                *presence = false;
                true
            } else {
                false
            }
        };

        if changed {
            self.trigger_presence_switch_event(false).await;
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Command timeout.")]
    Timeout,
    #[error("MQTT error: {0}")]
    Mqtt(#[from] Box<MqttError>),
    #[error("Channel dropped unexpectedly")]
    Recv,
    #[error("Invalid JSON data: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("State not found for device {0}")]
    StateNotFound(Z2mDeviceId),
    #[error("Not initialized")]
    NotInitialized,
    #[error("Cannot perform boolean operation: {0}")]
    Boolean(#[from] BooleanError),
}
