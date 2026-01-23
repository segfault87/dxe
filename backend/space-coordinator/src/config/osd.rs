use std::collections::HashMap;

use chrono::TimeDelta;
use dxe_types::UnitId;
use serde::Deserialize;

use crate::services::mqtt::MqttTopicPrefix;
use crate::tasks::osd_controller::types::{AlertData, MixerChannelData, MixerGlobalData};
use crate::types::AlertId;
use crate::utils::deserializers::deserialize_time_delta_seconds;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum AlertKind {
    OnSignOn,
    OnSignOff {
        #[serde(
            rename = "before_seconds",
            deserialize_with = "deserialize_time_delta_seconds"
        )]
        before: TimeDelta,
    },
    Alert {
        alert_id: AlertId,
    },
}

#[derive(Deserialize, Clone, Debug)]
pub struct AlertConfig {
    #[serde(flatten)]
    pub kind: AlertKind,
    pub unit_id: Option<UnitId>,
    #[serde(flatten)]
    pub data: AlertData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MixerConfig {
    pub channels: HashMap<i8, MixerChannelData>,
    pub globals: MixerGlobalData,
    #[serde(rename = "reset_after_seconds", deserialize_with = "deserialize_time_delta_seconds")]
    pub reset_after: TimeDelta,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub topic_prefix: MqttTopicPrefix,
    pub alerts: Vec<AlertConfig>,
    pub mixers: HashMap<UnitId, MixerConfig>,
    pub doorbell_alert_id: Option<AlertId>,
}
