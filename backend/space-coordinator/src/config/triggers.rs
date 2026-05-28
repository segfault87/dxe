use std::collections::{HashMap, HashSet};
use std::time::Duration;

use serde::Deserialize;

use crate::device::SwitchState;
use crate::types::{DeviceRef, EndpointKey, EventId};
use crate::utils::boolean::Expression;
use crate::utils::deserializers::deserialize_duration_milliseconds;

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateAction {
    Start,
    Stop,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerAction {
    #[serde(rename = "delay_milliseconds")]
    Delay(#[serde(deserialize_with = "deserialize_duration_milliseconds")] Duration),
    Switches(HashMap<DeviceRef, SwitchState>),
    BookingControl(StateAction),
    OsdControl(StateAction),
    BookingReminder,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Trigger {
    pub event_ids: HashSet<EventId>,
    #[serde(default)]
    pub condition: Option<Expression<EndpointKey>>,
    pub actions: Vec<TriggerAction>,
}
