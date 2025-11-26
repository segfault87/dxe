use std::collections::HashMap;

use dxe_types::UnitId;
use serde::Deserialize;

use crate::services::mqtt::DeviceName;

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresencePolicy {
    Default,
    StayOn,
}

impl Default for PresencePolicy {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeviceClassPowerMeter {}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwitchState {
    On,
    Off,
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Condition {
    pub state: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SwitchStates {
    pub on: Vec<serde_json::Value>,
    pub off: Vec<serde_json::Value>,
}

impl Default for SwitchStates {
    fn default() -> Self {
        Self {
            on: vec![serde_json::json!({"state": "ON"})],
            off: vec![serde_json::json!({"state": "OFF"})],
        }
    }
}

fn default_switch_condition() -> Vec<Condition> {
    vec![Condition {
        state: String::from("state"),
        operator: ConditionOperator::Eq,
        value: serde_json::json!("ON"),
    }]
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeviceClassSwitch {
    #[serde(default)]
    pub presence_policy: PresencePolicy,
    #[serde(default)]
    pub states: SwitchStates,
    #[serde(default = "default_switch_condition")]
    pub is_on: Vec<Condition>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeviceClasses {
    pub power_meter: Option<DeviceClassPowerMeter>,
    pub switch: Option<DeviceClassSwitch>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Device {
    pub name: DeviceName,
    pub state_key: String,
    pub classes: DeviceClasses,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Hook {
    #[serde(default)]
    pub switches: HashMap<DeviceName, SwitchState>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PerUnitHooks {
    #[serde(default)]
    pub on_booking_start_buffered: Hook,
    #[serde(default)]
    pub on_booking_start: Hook,
    #[serde(default)]
    pub on_booking_end: Hook,
    #[serde(default)]
    pub on_booking_buffered: Hook,
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

#[derive(Clone, Debug, Deserialize)]
pub struct Alert {
    pub name: String,
    pub device: DeviceName,
    pub conditions: Vec<Condition>,
    pub presence: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub devices: Vec<Device>,
    pub hooks: Hooks,
    pub alerts: Vec<Alert>,
}
