use std::collections::HashMap;
use std::net::IpAddr;

use chrono::TimeDelta;
use serde::Deserialize;

use crate::types::TenantId;
use crate::utils::deserializers::deserialize_time_delta_seconds;

#[derive(Clone, Debug, Deserialize)]
pub struct PresenceIdentityConfig {
    pub scan_ips: Vec<IpAddr>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub identities: HashMap<TenantId, PresenceIdentityConfig>,
    #[serde(
        rename = "away_interval_secs",
        deserialize_with = "deserialize_time_delta_seconds"
    )]
    pub away_interval: TimeDelta,
}
