use std::collections::{HashMap, HashSet};

use dxe_types::entities::{MixerChannelData, MixerGlobalData};
use dxe_types::{MixerChannelId, UnitId};
use serde::{Deserialize, Serialize};

use crate::services::mqtt::MqttTopicPrefix;
use crate::tasks::osd_controller::types::{AlertData, MixerPresets};
use crate::types::EventId;

#[derive(Deserialize, Clone, Debug)]
pub struct AlertConfig {
    pub event_ids: HashSet<EventId>,
    #[serde(flatten)]
    pub data: AlertData,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MixerConfig {
    pub channels: HashMap<MixerChannelId, MixerChannelData>,
    pub globals: MixerGlobalData,
    #[serde(default)]
    pub export: bool,
}

impl From<MixerConfig> for MixerPresets {
    fn from(value: MixerConfig) -> Self {
        Self {
            channels: value
                .channels
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            globals: value.globals.into(),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub topic_prefix: MqttTopicPrefix,
    pub alerts: Vec<AlertConfig>,
    pub mixers: HashMap<UnitId, MixerConfig>,
    pub doorbell_event_id: Option<EventId>,
}
