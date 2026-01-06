use chrono::TimeDelta;
use serde::Deserialize;

use crate::types::{DeviceRef, MetricId, PublishKey};
use crate::utils::deserializers::deserialize_time_delta_milliseconds_optional;

#[derive(Clone, Debug, Deserialize)]
pub struct Metric {
    pub id: MetricId,
    #[serde(
        rename = "average_window_milliseconds",
        deserialize_with = "deserialize_time_delta_milliseconds_optional",
        default
    )]
    pub average_window: Option<TimeDelta>,
    pub devices: Vec<DeviceRef>,
    pub publish_keys: Vec<PublishKey>,
}
