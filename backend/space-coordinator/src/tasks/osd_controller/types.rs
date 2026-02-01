use std::collections::HashMap;

use chrono::{DateTime, Utc};
use dxe_types::BookingId;
use dxe_types::IdentityId;
use dxe_types::entities;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlertSeverity {
    Urgent,
    Normal,
    Intrusive,
}

fn default_closeable() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AlertData {
    pub severity: AlertSeverity,
    pub title: String,
    pub contents: String,
    #[serde(default = "default_closeable")]
    pub closeable: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Booking {
    pub booking_id: BookingId,
    pub customer_id: IdentityId,
    pub customer_name: String,
    pub time_from: DateTime<Utc>,
    pub time_to: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParkingState {
    pub license_plate_number: String,
    pub user_name: String,
    pub entry_date: DateTime<Utc>,
    pub exempted: bool,
    pub fuzzy: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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

impl From<MixerChannelData> for entities::MixerChannelData {
    fn from(value: MixerChannelData) -> Self {
        Self {
            level: value.level,
            pan: value.pan,
            reverb: value.reverb,
            mute: value.mute,
            eq_high_level: value.eq_high_level,
            eq_high_freq: value.eq_high_freq,
            eq_mid_level: value.eq_mid_level,
            eq_mid_freq: value.eq_mid_freq,
            eq_mid_q: value.eq_mid_q,
            eq_low_level: value.eq_low_level,
            eq_low_freq: value.eq_low_freq,
        }
    }
}

impl From<entities::MixerChannelData> for MixerChannelData {
    fn from(value: entities::MixerChannelData) -> Self {
        Self {
            level: value.level,
            pan: value.pan,
            reverb: value.reverb,
            mute: value.mute,
            eq_high_level: value.eq_high_level,
            eq_high_freq: value.eq_high_freq,
            eq_mid_level: value.eq_mid_level,
            eq_mid_freq: value.eq_mid_freq,
            eq_mid_q: value.eq_mid_q,
            eq_low_level: value.eq_low_level,
            eq_low_freq: value.eq_low_freq,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MixerGlobalData {
    pub master_level: Option<f64>,
    pub monitor_level: Option<f64>,
}

impl From<MixerGlobalData> for entities::MixerGlobalData {
    fn from(value: MixerGlobalData) -> Self {
        Self {
            master_level: value.master_level,
            monitor_level: value.monitor_level,
        }
    }
}

impl From<entities::MixerGlobalData> for MixerGlobalData {
    fn from(value: entities::MixerGlobalData) -> Self {
        Self {
            master_level: value.master_level,
            monitor_level: value.monitor_level,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MixerPresets {
    pub channels: Vec<MixerChannelData>,
    pub globals: MixerGlobalData,
}

impl From<MixerPresets> for entities::MixerPresets {
    fn from(value: MixerPresets) -> Self {
        Self {
            channels: value.channels.into_iter().map(Into::into).collect(),
            globals: value.globals.into(),
        }
    }
}

impl From<entities::MixerPresets> for MixerPresets {
    fn from(value: entities::MixerPresets) -> Self {
        Self {
            channels: value.channels.into_iter().map(Into::into).collect(),
            globals: value.globals.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MixerPreferences {
    pub default: MixerPresets,
    pub scenes: HashMap<String, MixerPresets>,
}

impl From<MixerPreferences> for entities::MixerPreferences {
    fn from(value: MixerPreferences) -> Self {
        Self {
            default: value.default.into(),
            scenes: value
                .scenes
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl From<entities::MixerPreferences> for MixerPreferences {
    fn from(value: entities::MixerPreferences) -> Self {
        Self {
            default: value.default.into(),
            scenes: value
                .scenes
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMixerConfig {
    pub customer_id: IdentityId,
    pub prefs: MixerPreferences,
}
