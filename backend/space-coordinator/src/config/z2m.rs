use std::collections::{HashMap, HashSet};

use chrono::TimeDelta;
use serde::Deserialize;

use crate::types::{PublishKey, PublishedValues, Z2mDeviceId};
use crate::utils::boolean::{ComparisonOperator, Condition, Expression};
use crate::utils::deserializers::deserialize_time_delta_seconds;

fn default_power_meter_state_keys() -> Vec<PublishKey> {
    vec![String::from("energy").into(), String::from("power").into()]
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeviceClassPowerMeter {
    #[serde(default = "default_power_meter_state_keys")]
    pub state_keys: Vec<PublishKey>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Z2mSwitchStates {
    pub on: Vec<PublishedValues>,
    pub off: Vec<PublishedValues>,
}

impl Default for Z2mSwitchStates {
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
    pub states: Z2mSwitchStates,
    #[serde(default = "default_switch_condition")]
    pub is_on: Expression<PublishKey>,
    #[serde(default = "default_switch_state_keys")]
    pub state_keys: Vec<PublishKey>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct DeviceClasses {
    pub power_meter: Option<DeviceClassPowerMeter>,
    pub switch: Option<DeviceClassSwitch>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Device {
    pub id: Z2mDeviceId,
    #[serde(default)]
    pub state_keys: HashSet<PublishKey>,
    #[serde(default)]
    pub volatile_state_keys: HashSet<PublishKey>,
    #[serde(default)]
    pub classes: DeviceClasses,
    #[serde(default)]
    pub skip_sync: bool,
}

impl Device {
    pub fn state_keys(&self) -> HashSet<PublishKey> {
        let mut keys = HashSet::new();

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

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(
        rename = "command_timeout_secs",
        deserialize_with = "deserialize_time_delta_seconds"
    )]
    pub command_timeout: TimeDelta,
    pub devices: Vec<Device>,
}
