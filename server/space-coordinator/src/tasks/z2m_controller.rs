use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_types::{BookingId, UnitId};
use rumqttc::Publish;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::callback::{EventStateCallback, PresenceCallback};
use crate::config::z2m::{self, SwitchPolicy};
use crate::services::mqtt::{Error as MqttError, MqttService};
use crate::services::notification::NotificationService;

#[derive(Debug)]
pub enum Z2mPublishTopic {
    Get,
    Set,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct DeviceName(String);

impl From<String> for DeviceName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl DeviceName {
    pub fn topic_name(&self, command: Option<Z2mPublishTopic>) -> String {
        format!(
            "zigbee2mqtt/{}{}",
            self.0,
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
            .map(|v| DeviceName(v.to_owned()))
    }
}

impl Display for DeviceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct PowerUsage {
    instant_wattage: f64,
    sum_kwh: f64,
}

pub struct Z2mController {
    mqtt_service: MqttService,
    notification_service: NotificationService,

    command_timeout: Duration,

    devices: HashMap<DeviceName, z2m::Device>,
    device_states: Mutex<HashMap<DeviceName, serde_json::Value>>,
    per_unit_hooks: HashMap<UnitId, z2m::PerUnitHooks>,
    presence_hooks: z2m::PresenceHooks,
    alerts: Vec<z2m::Alert>,

    active_bookings: Mutex<Option<HashMap<UnitId, HashSet<BookingId>>>>,
    presence: Mutex<Option<bool>>,
    alert_flags: Mutex<HashSet<String>>,
}

fn test_condition(
    value: &serde_json::Value,
    conditions: &Vec<z2m::Condition>,
) -> Result<bool, Error> {
    let serde_json::Value::Object(value) = value else {
        return Err(Error::Json);
    };

    for condition in conditions {
        let Some(value) = value.get(&condition.state) else {
            return Err(Error::StateVariableNotFound(condition.state.clone()));
        };

        let result = match condition.operator {
            z2m::ConditionOperator::Eq => value == &condition.value,
            z2m::ConditionOperator::Ne => value != &condition.value,
            _ => {
                let serde_json::Value::Number(lhs) = value else {
                    return Err(Error::Arithmetic(value.clone()));
                };
                let serde_json::Value::Number(rhs) = &condition.value else {
                    return Err(Error::Arithmetic(value.clone()));
                };

                if let Some(lhs) = lhs.as_i64() {
                    let rhs = rhs
                        .as_i64()
                        .or(rhs.as_f64().map(|v| v as i64))
                        .ok_or(Error::Arithmetic(condition.value.clone()))?;

                    match condition.operator {
                        z2m::ConditionOperator::Gt => lhs > rhs,
                        z2m::ConditionOperator::Ge => lhs >= rhs,
                        z2m::ConditionOperator::Lt => lhs < rhs,
                        z2m::ConditionOperator::Le => lhs <= rhs,
                        _ => unreachable!(),
                    }
                } else if let Some(lhs) = lhs.as_f64() {
                    let rhs = rhs
                        .as_f64()
                        .or(rhs.as_i64().map(|v| v as f64))
                        .ok_or(Error::Arithmetic(condition.value.clone()))?;

                    match condition.operator {
                        z2m::ConditionOperator::Gt => lhs > rhs,
                        z2m::ConditionOperator::Ge => lhs >= rhs,
                        z2m::ConditionOperator::Lt => lhs < rhs,
                        z2m::ConditionOperator::Le => lhs <= rhs,
                        _ => unreachable!(),
                    }
                } else {
                    return Err(Error::Arithmetic(value.clone()));
                }
            }
        };

        if !result {
            return Ok(false);
        }
    }

    Ok(true)
}

impl Z2mController {
    pub fn new(
        config: &z2m::Config,
        mqtt_service: MqttService,
        notification_service: NotificationService,
    ) -> Self {
        Self {
            mqtt_service,
            notification_service,

            command_timeout: Duration::from_secs(config.command_timeout_secs),

            devices: config
                .devices
                .iter()
                .map(|v| (v.name.clone(), v.clone()))
                .collect(),
            device_states: Mutex::new(HashMap::new()),
            per_unit_hooks: config.hooks.units.clone(),
            presence_hooks: config.hooks.presence.clone(),
            alerts: config.alerts.clone(),

            active_bookings: Mutex::new(Default::default()),
            presence: Mutex::new(None),
            alert_flags: Mutex::new(HashSet::new()),
        }
    }

    pub async fn start(&mut self) {
        let mut device_count = self.devices.len();
        for (name, device) in self.devices.iter() {
            if let Err(e) = self.mqtt_service.subscribe(&name.topic_name(None)).await {
                log::warn!("Cannot subscribe to {name}: {e}");
                continue;
            }

            let value = match self.get_state(device).await {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Could not get state for device {name}: {e}");
                    continue;
                }
            };

            log::info!("Got initial state for {name}");
            device_count -= 1;

            self.device_states
                .lock()
                .unwrap()
                .insert(name.clone(), value);
        }

        if device_count == 0 {
            log::info!("Synchronized z2m devices successfully.");
        } else {
            log::warn!("{device_count} devices were not synchronized. Proceeding...");
        }
    }

    async fn get_state(&self, device: &z2m::Device) -> Result<serde_json::Value, Error> {
        let state_key = &device.state_key;
        let state_key = json!({state_key: {}});

        let mut receiver = self.mqtt_service.receiver();

        let (tx, rx) = oneshot::channel();

        let expected_device_name = device.name.clone();

        tokio::task::spawn(async move {
            loop {
                let Ok(publish) = receiver.recv().await else {
                    break;
                };

                if let Some(device_name) = DeviceName::from_topic_name(&publish.topic) {
                    if expected_device_name == device_name {
                        let payload =
                            match serde_json::from_slice::<serde_json::Value>(&publish.payload) {
                                Ok(v) => v,
                                Err(e) => {
                                    log::warn!(
                                        "Invalid JSON string from topic {}: {e}",
                                        publish.topic
                                    );
                                    continue;
                                }
                            };

                        let _ = tx.send(payload);
                        break;
                    }
                }
            }
        });

        self.mqtt_service
            .publish(
                &device.name.topic_name(Some(Z2mPublishTopic::Get)),
                state_key.to_string().as_bytes(),
            )
            .await
            .map_err(|e| Error::Mqtt(Box::new(e)))?;

        match timeout(self.command_timeout, rx).await {
            Ok(v) => Ok(v.map_err(|_| Error::Recv)?),
            Err(_) => Err(Error::Timeout),
        }
    }

    pub async fn set_state(
        &self,
        name: DeviceName,
        states: &[serde_json::Value],
    ) -> Result<(), Error> {
        let mut receiver = self.mqtt_service.receiver();

        let (tx, rx) = oneshot::channel();

        let states_to_inspect: Result<Vec<serde_json::Map<String, serde_json::Value>>, _> = states
            .iter()
            .map(|v| {
                serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(v.clone())
            })
            .collect();
        let mut states_to_inspect = states_to_inspect?;
        if states_to_inspect.is_empty() {
            return Ok(());
        }

        let mqtt_service = self.mqtt_service.clone();

        tokio::task::spawn(async move {
            if let Err(e) = mqtt_service
                .publish(
                    &name.topic_name(Some(Z2mPublishTopic::Set)),
                    serde_json::Value::Object(states_to_inspect.first().unwrap().clone())
                        .to_string()
                        .as_bytes(),
                )
                .await
            {
                log::warn!("Could not publish set command: {e}");
                return;
            }

            let mut first = states_to_inspect.first().unwrap();
            loop {
                let Ok(publish) = receiver.recv().await else {
                    break;
                };

                if let Some(device_name) = DeviceName::from_topic_name(&publish.topic) {
                    if name == device_name {
                        let payload = match serde_json::from_slice::<
                            serde_json::Map<String, serde_json::Value>,
                        >(&publish.payload)
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
                            if let Err(e) = mqtt_service
                                .publish(
                                    &name.topic_name(Some(Z2mPublishTopic::Set)),
                                    serde_json::Value::Object(
                                        states_to_inspect.first().unwrap().clone(),
                                    )
                                    .to_string()
                                    .as_bytes(),
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
            }
        });

        match timeout(self.command_timeout, rx).await {
            Ok(v) => Ok(v.map_err(|_| Error::Recv)?),
            Err(_) => Err(Error::Timeout),
        }
    }

    pub fn handle_publishment(self: Arc<Self>, publish: Publish) {
        if let Some(device_name) = DeviceName::from_topic_name(&publish.topic)
            && self.devices.contains_key(&device_name)
        {
            let value = match serde_json::from_slice::<serde_json::Value>(&publish.payload) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Failed to serialize payload for {device_name}: {e}");
                    return;
                }
            };

            self.device_states
                .lock()
                .unwrap()
                .insert(device_name, value);
        }
    }

    fn get_switch_state(
        &self,
        device_name: &DeviceName,
        switch: &z2m::DeviceClassSwitch,
    ) -> Result<z2m::SwitchState, Error> {
        let device_states = self.device_states.lock().unwrap();

        let Some(state) = device_states.get(device_name) else {
            return Err(Error::StateNotFound(device_name.clone()));
        };

        test_condition(state, &switch.is_on).map(|v| {
            if v {
                z2m::SwitchState::On
            } else {
                z2m::SwitchState::Off
            }
        })
    }

    fn is_active_bookings(&self) -> Option<bool> {
        let guard = self.active_bookings.lock().unwrap();

        Some(guard.as_ref()?.values().any(|v| !v.is_empty()))
    }

    fn get_active_units(&self) -> Vec<UnitId> {
        let guard = self.active_bookings.lock().unwrap();

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
        device_name: &DeviceName,
        switch: &z2m::DeviceClassSwitch,
        state: z2m::SwitchState,
    ) -> Result<bool, Error> {
        let current_state = self.get_switch_state(device_name, switch)?;

        if current_state == state {
            Ok(false)
        } else {
            log::info!("Setting switch {device_name} to {state}");

            self.set_state(
                device_name.clone(),
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
        device_name: &DeviceName,
        switch: &z2m::DeviceClassSwitch,
    ) -> Result<bool, Error> {
        let current_state = self.get_switch_state(device_name, switch)?;

        let Some(is_present) = *self.presence.lock().unwrap() else {
            return Err(Error::NotInitialized);
        };
        let Some(is_active_bookings) = self.is_active_bookings() else {
            return Err(Error::NotInitialized);
        };

        let switch_policy = if is_present {
            switch.presence_policy
        } else if is_active_bookings {
            switch.booking_policy
        } else {
            z2m::SwitchPolicy::Off
        };

        let desired_state = match switch_policy {
            SwitchPolicy::Uncontrolled => return Ok(false),
            SwitchPolicy::StayOn => z2m::SwitchState::On,
            SwitchPolicy::Off => z2m::SwitchState::Off,
        };

        if current_state == desired_state {
            Ok(false)
        } else {
            log::info!("Setting switch {device_name} to {desired_state}");

            self.set_state(
                device_name.clone(),
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
            if let Some(switch) = &device.classes.switch {
                if let Err(e) = self.sync_switch_state(&device.name, switch).await {
                    log::warn!("Failed to sync switch state for {}: {e}", device.name);
                }
            }
        }
    }

    fn aggregate_switch_states(
        states: &mut HashMap<DeviceName, z2m::SwitchState>,
        existing_states: &HashMap<DeviceName, z2m::SwitchState>,
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

        for (device_name, state) in switches {
            let Some(switch) = self
                .devices
                .get(&device_name)
                .and_then(|v| v.classes.switch.as_ref())
            else {
                log::warn!("Device {device_name} is not a switch.");
                continue;
            };

            if let Err(e) = self.set_switch(&device_name, switch, state).await {
                log::warn!("Could not change switch state of {device_name} to {state}: {e}");
            }
        }
    }

    async fn trigger_unit_event(&self, unit_id: &UnitId, start: bool) {
        let Some(hook) = self.per_unit_hooks.get(unit_id) else {
            return;
        };

        let Some(presence) = *self.presence.lock().unwrap() else {
            return;
        };

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

        for (device_name, state) in switches {
            let Some(switch) = self
                .devices
                .get(&device_name)
                .and_then(|v| v.classes.switch.as_ref())
            else {
                log::warn!("Device {device_name} is not a switch.");
                continue;
            };

            if let Err(e) = self.set_switch(&device_name, switch, state).await {
                log::warn!("Could not change switch state of {device_name} to {state}: {e}");
            }
        }
    }

    fn read_power_meter(&self, device: &z2m::Device) -> Result<PowerUsage, Error> {
        let Some(_power_meter) = &device.classes.power_meter else {
            return Err(Error::ClassMismatch("power_meter", device.name.clone()));
        };

        let device_states = self.device_states.lock().unwrap();
        let Some(state) = device_states.get(&device.name) else {
            return Err(Error::StateNotFound(device.name.clone()));
        };

        let state = state.as_object().ok_or(Error::Json)?;

        let energy = state
            .get("energy")
            .and_then(|v| v.as_f64())
            .unwrap_or_default();
        let power = state
            .get("power")
            .and_then(|v| v.as_f64())
            .unwrap_or_default();

        Ok(PowerUsage {
            instant_wattage: power,
            sum_kwh: energy,
        })
    }

    async fn handle_alert_tasks(&self) {
        let Some(presence) = *self.presence.lock().unwrap() else {
            return;
        };

        let Some(booking_status) = self.is_active_bookings() else {
            return;
        };

        for alert in self.alerts.iter() {
            if self.alert_flags.lock().unwrap().contains(&alert.name) {
                continue;
            }

            if let Some(alert_presence) = alert.presence
                && alert_presence != presence
            {
                continue;
            }
            if let Some(booking) = alert.booking
                && booking_status != booking
            {
                continue;
            }

            let Some(device) = self.devices.get(&alert.device) else {
                log::warn!(
                    "Device {} not found for alert condition \"{}\"",
                    alert.device,
                    alert.name
                );
                continue;
            };

            let Some(state) = self
                .device_states
                .lock()
                .unwrap()
                .get(&device.name)
                .cloned()
            else {
                log::warn!("Device state not found for {}", device.name);
                continue;
            };

            match test_condition(&state, &alert.conditions) {
                Ok(true) => {
                    self.alert_flags.lock().unwrap().insert(alert.name.clone());
                    if let Err(e) = self
                        .notification_service
                        .notify(alert.priority, alert.name.clone())
                        .await
                    {
                        log::warn!("Could not send alert for \"{}\": {e}", alert.name);
                    }
                }
                Ok(false) => {}
                Err(e) => {
                    log::warn!("Could not test alert condition for \"{}\": {e}", alert.name);
                }
            }
        }
    }

    async fn update(self: Arc<Self>) {
        if self.active_bookings.lock().unwrap().is_none() || self.presence.lock().unwrap().is_none()
        {
            return;
        }

        self.handle_alert_tasks().await;
        self.sync_switch_states().await;
    }

    pub fn task(self) -> (Arc<Self>, JoinHandle<()>, Task) {
        let task_name = "mqtt_controller".to_string();

        let mut receiver = self.mqtt_service.receiver();

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
        *self.active_bookings.lock().unwrap() = Some(Default::default());
    }

    async fn on_event_start(
        &self,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if buffered {
            {
                let mut guard = self.active_bookings.lock().unwrap();
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
        &self,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if buffered {
            {
                let mut guard = self.active_bookings.lock().unwrap();
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
        *self.presence.lock().unwrap() = Some(true);

        self.alert_flags.lock().unwrap().clear();

        self.trigger_presence_switch_event(true).await;

        Ok(())
    }

    async fn on_leave(&self) -> Result<(), Box<dyn StdError>> {
        *self.presence.lock().unwrap() = Some(false);

        self.trigger_presence_switch_event(false).await;

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
    #[error("Invalid JSON data.")]
    Json,
    #[error("Invalid JSON data: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Z2m state variable {0} not found.")]
    StateVariableNotFound(String),
    #[error("Arithmetic operator is applicable on numerals. value: {0}")]
    Arithmetic(serde_json::Value),
    #[error("No device class {0} for device: {1}")]
    ClassMismatch(&'static str, DeviceName),
    #[error("State not found for device {0}")]
    StateNotFound(DeviceName),
    #[error("Not initialized")]
    NotInitialized,
}
