use dxe_types::UnitId;
use serde::Serialize;

use super::OsdTopic;
use super::types::{AlertData, Booking, MixerPreferences, MixerPresets, ParkingState};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    #[serde(skip)]
    pub unit_id: UnitId,
    pub alert: Option<AlertData>,
}

impl OsdTopic for Alert {
    fn topic_name(&self) -> String {
        format!("alert/{}", self.unit_id)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParkingStates {
    #[serde(skip)]
    pub unit_id: UnitId,
    pub states: Vec<ParkingState>,
}

impl OsdTopic for ParkingStates {
    fn topic_name(&self) -> String {
        format!("parking_states/{}", self.unit_id)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetScreenState {
    #[serde(skip)]
    pub unit_id: UnitId,
    pub is_active: bool,
}

impl OsdTopic for SetScreenState {
    fn topic_name(&self) -> String {
        format!("screen_state/{}", self.unit_id)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentSession {
    #[serde(skip)]
    pub unit_id: UnitId,
    pub booking: Option<Booking>,
}

impl OsdTopic for CurrentSession {
    fn topic_name(&self) -> String {
        format!("current_session/{}", self.unit_id)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoorLockOpenResult {
    pub success: bool,
    pub error: Option<String>,
}

impl OsdTopic for DoorLockOpenResult {
    fn topic_name(&self) -> String {
        String::from("doorlock/get")
    }
}

// To be used later
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SetMixerStates {
    #[serde(skip)]
    pub unit_id: UnitId,
    pub overwrite: bool,
    pub state: MixerPresets,
}

impl OsdTopic for SetMixerStates {
    fn topic_name(&self) -> String {
        format!("mixer_states/{}", self.unit_id)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetMixerPreferences {
    #[serde(skip)]
    pub unit_id: UnitId,
    pub prefs: MixerPreferences,
}

impl OsdTopic for SetMixerPreferences {
    fn topic_name(&self) -> String {
        format!("mixer_preferences/{}", self.unit_id)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoorbellRequest {
    pub unit_id: Option<UnitId>,
}

impl OsdTopic for DoorbellRequest {
    fn topic_name(&self) -> String {
        String::from("doorbell_request")
    }
}
