use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MixerChannelData {
    pub level: Option<f64>,
    pub pan: Option<f64>,
    pub reverb: Option<f64>,
    pub mute: Option<bool>,
    pub eq_high_level: Option<f64>,
    pub eq_high_freq: Option<f64>,
    pub eq_mid_level: Option<f64>,
    pub eq_mid_freq: Option<f64>,
    pub eq_mid_q: Option<f64>,
    pub eq_low_level: Option<f64>,
    pub eq_low_freq: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MixerGlobalData {
    pub master_level: Option<f64>,
    pub monitor_level: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MixerPresets {
    pub channels: Vec<MixerChannelData>,
    pub globals: MixerGlobalData,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MixerPreferences {
    pub default: MixerPresets,
    pub scenes: HashMap<String, MixerPresets>,
}
