use std::{collections::HashMap, fmt::Display};

use chrono::TimeDelta;
use dxe_types::UnitId;
use serde::Deserialize;

use crate::types::{PublishKey, PublishedValues, Z2mDeviceId};
use crate::utils::boolean::{ComparisonOperator, Condition, Expression};
use crate::utils::deserializers::deserialize_time_delta_seconds;

#[derive(Copy, Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SwitchPolicy {
    #[default]
    Uncontrolled,
    StayOn,
    Off,
}

fn default_power_meter_state_keys() -> Vec<PublishKey> {
    vec![String::from("energy").into(), String::from("power").into()]
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeviceClassPowerMeter {
    #[serde(default = "default_power_meter_state_keys")]
    pub state_keys: Vec<PublishKey>,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SwitchState {
    On,
    Off,
}

impl Display for SwitchState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::On => write!(f, "ON"),
            Self::Off => write!(f, "OFF"),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SwitchStates {
    pub on: Vec<PublishedValues>,
    pub off: Vec<PublishedValues>,
}

impl Default for SwitchStates {
    fn default() -> Self {
        Self {
            on: vec![HashMap::from([(
                String::from("state").into(),
                serde_json::json!("ON"),
            )])],
            off: vec![HashMap::from([(
                String::from("state").into(),
                serde_json::json!("OFF"),
            )])],
        }
    }
}

fn default_switch_condition() -> Expression<PublishKey> {
    Expression::Unary(Condition {
        key: String::from("state").into(),
        operator: ComparisonOperator::Eq,
        value: serde_json::Value::String(String::from("ON")),
    })
}

fn default_switch_state_keys() -> Vec<PublishKey> {
    vec![String::from("state").into()]
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeviceClassSwitch {
    #[serde(default)]
    pub presence_policy: SwitchPolicy,
    #[serde(default)]
    pub booking_policy: SwitchPolicy,
    #[serde(default)]
    pub states: SwitchStates,
    #[serde(default = "default_switch_condition")]
    pub is_on: Expression<PublishKey>,
    #[serde(default = "default_switch_state_keys")]
    pub state_keys: Vec<PublishKey>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeviceClasses {
    pub power_meter: Option<DeviceClassPowerMeter>,
    pub switch: Option<DeviceClassSwitch>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Device {
    pub id: Z2mDeviceId,
    #[serde(default)]
    pub state_keys: Vec<PublishKey>,
    pub classes: DeviceClasses,
}

impl Device {
    pub fn state_keys(&self) -> Vec<PublishKey> {
        let mut keys = vec![];

        keys.extend(self.state_keys.iter().cloned());
        if let Some(switch) = &self.classes.switch {
            keys.extend(switch.state_keys.iter().cloned());
        }
        if let Some(power_meter) = &self.classes.power_meter {
            keys.extend(power_meter.state_keys.iter().cloned());
        }

        keys
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Hook {
    #[serde(default)]
    pub switches: HashMap<Z2mDeviceId, SwitchState>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PerUnitHooks {
    #[serde(default)]
    pub on_booking_start: Hook,
    #[serde(default)]
    pub on_booking_end: Hook,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PresenceHooks {
    #[serde(default)]
    pub on_enter: Hook,
    #[serde(default)]
    pub on_leave: Hook,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Hooks {
    pub units: HashMap<UnitId, PerUnitHooks>,
    pub presence: PresenceHooks,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(
        rename = "command_timeout_secs",
        deserialize_with = "deserialize_time_delta_seconds"
    )]
    pub command_timeout: TimeDelta,
    pub devices: Vec<Device>,
    pub hooks: Hooks,
}
