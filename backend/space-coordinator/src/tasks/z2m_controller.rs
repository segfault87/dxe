use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::TimeDelta;
use rumqttc::Publish;
use serde_json::json;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::config::z2m;
use crate::device::SwitchState;
use crate::services::mqtt::{Error as MqttError, MqttService, MqttTopicPrefix};
use crate::tables::{QualifiedPath, SingleTable, TablePublisher};
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

    state_keys: HashMap<Z2mDeviceId, HashSet<PublishKey>>,
    volatile_state_keys: HashMap<Z2mDeviceId, HashSet<PublishKey>>,
    table: TablePublisher<DeviceId, DeviceRef, Z2mPath>,
}

impl Z2mController {
    pub fn new(config: &z2m::Config, mqtt_service: MqttService) -> Self {
        Self {
            mqtt_service,

            command_timeout: config.command_timeout,

            devices: config
                .devices
                .iter()
                .map(|v| (v.id.clone(), v.clone()))
                .collect(),

            state_keys: config
                .devices
                .iter()
                .map(|v| (v.id.clone(), v.state_keys()))
                .collect(),
            volatile_state_keys: config
                .devices
                .iter()
                .map(|v| (v.id.clone(), v.volatile_state_keys.clone()))
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
            let mut values = match serde_json::from_slice::<PublishedValues>(&publish.payload) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Failed to serialize payload for {device_id}: {e}");
                    return;
                }
            };

            if let Some(state_keys) = self.state_keys.get(&device_id) {
                let states = state_keys
                    .iter()
                    .filter_map(|key| values.remove(key).map(|v| (key.clone(), v)))
                    .collect::<HashMap<_, _>>();

                if !states.is_empty() {
                    self.table.update(device.id.clone().into(), states);
                }
            }

            if let Some(volatile_state_keys) = self.volatile_state_keys.get(&device_id) {
                let states = volatile_state_keys
                    .iter()
                    .filter_map(|key| values.remove(key).map(|v| (key.clone(), v)))
                    .collect::<HashMap<_, _>>();

                if !states.is_empty() {
                    let clearance = states
                        .keys()
                        .map(|v| (v.clone(), serde_json::Value::Null))
                        .collect::<HashMap<_, _>>();

                    self.table.update(device.id.clone().into(), states);
                    // Clear immediately after broadcasting
                    self.table.update(device.id.clone().into(), clearance);
                }
            }
        }
    }

    fn get_switch_state(
        &self,
        device_id: Z2mDeviceId,
        switch: &z2m::DeviceClassSwitch,
    ) -> Result<SwitchState, Error> {
        let Some(state) = self.table.get_values(&device_id.clone().into()) else {
            return Err(Error::StateNotFound(device_id));
        };

        if switch.is_on.test(&state)? {
            Ok(SwitchState::On)
        } else {
            Ok(SwitchState::Off)
        }
    }

    pub async fn set_switch(
        &self,
        device_id: Z2mDeviceId,
        state: SwitchState,
    ) -> Result<bool, Error> {
        let device = self
            .devices
            .get(&device_id)
            .ok_or(Error::StateNotFound(device_id.clone()))?;

        let Some(switch) = &device.classes.switch else {
            return Err(Error::DeviceNotCompliant(device_id.clone(), "switch"));
        };

        let current_state = self.get_switch_state(device_id.clone(), switch)?;

        if current_state == state {
            Ok(false)
        } else {
            log::info!("Setting switch {device_id} to {state}");

            self.set_state(
                device_id.clone(),
                match state {
                    SwitchState::On => &switch.states.on,
                    SwitchState::Off => &switch.states.off,
                },
            )
            .await?;
            Ok(true)
        }
    }

    async fn update(self: Arc<Self>) {
        // TODO
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
    #[error("Cannot perform boolean operation: {0}")]
    Boolean(#[from] BooleanError),
    #[error("Device {0} is not compliant with {1} class")]
    DeviceNotCompliant(Z2mDeviceId, &'static str),
}
