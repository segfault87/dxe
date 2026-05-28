use std::collections::{HashMap, HashSet};

use dxe_types::UnitId;
use dxe_types::entities::MixerPresets;
use serde::Deserialize;

use crate::services::mqtt::MqttTopicPrefix;
use crate::tasks::osd_controller::types::AlertData;
use crate::types::EventId;

#[derive(Deserialize, Clone, Debug)]
pub struct AlertConfig {
    pub event_ids: HashSet<EventId>,
    #[serde(flatten)]
    pub data: AlertData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub topic_prefix: MqttTopicPrefix,
    pub alerts: Vec<AlertConfig>,
    pub mixers: HashMap<UnitId, MixerPresets>,
    pub doorbell_event_id: Option<EventId>,
}
