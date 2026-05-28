use std::collections::{HashMap, HashSet};

use chrono::TimeDelta;
use dxe_types::UnitId;
use serde::Deserialize;

use crate::types::{AlertId, BookingEventId, EndpointKey};
use crate::utils::boolean::Expression;
use crate::utils::deserializers::{
    deserialize_time_delta_milliseconds_optional, deserialize_time_delta_seconds,
    deserialize_time_delta_seconds_optional,
};

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BookingEventType {
    OnStart,
    OnEnd,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BookingEventConfig {
    pub unit_ids: Vec<UnitId>,
    pub r#type: BookingEventType,
    #[serde(
        deserialize_with = "deserialize_time_delta_seconds",
        rename = "offset_secs",
        default
    )]
    pub offset: TimeDelta,
    #[serde(default)]
    pub continuation: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AlertConfig {
    pub condition: Expression<EndpointKey>,
    #[serde(
        rename = "debounce_milliseconds",
        deserialize_with = "deserialize_time_delta_seconds_optional",
        default
    )]
    pub debounce: Option<TimeDelta>,
    #[serde(
        rename = "grace_milliseconds",
        deserialize_with = "deserialize_time_delta_milliseconds_optional",
        default
    )]
    pub grace: Option<TimeDelta>,
    #[serde(default)]
    pub unit_ids: Option<HashSet<UnitId>>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub bookings: HashMap<BookingEventId, BookingEventConfig>,
    #[serde(default)]
    pub alerts: HashMap<AlertId, AlertConfig>,
}
