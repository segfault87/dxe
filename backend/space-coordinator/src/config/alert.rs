use chrono::TimeDelta;
use dxe_types::UnitId;
use serde::Deserialize;

use crate::types::{AlertId, EndpointRef};
use crate::utils::boolean::Expression;
use crate::utils::deserializers::deserialize_time_delta_seconds_optional;

#[derive(Clone, Debug, Deserialize)]
pub struct Alert {
    pub id: AlertId,
    pub condition: Expression<EndpointRef>,
    pub presence: Option<bool>,
    pub bookings: Option<Vec<UnitId>>,
    #[serde(
        rename = "snooze_secs",
        deserialize_with = "deserialize_time_delta_seconds_optional"
    )]
    pub snooze: Option<TimeDelta>,
}
