use dxe_types::{BookingId, UnitId};
use serde::Serialize;

use super::OsdTopic;
use super::types::{AlertSeverity, Booking, ParkingState};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    #[serde(skip)]
    pub unit_id: UnitId,
    pub severity: AlertSeverity,
    pub message: String,
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
pub struct StartSession {
    #[serde(skip)]
    pub unit_id: UnitId,
    pub booking: Booking,
}

impl OsdTopic for StartSession {
    fn topic_name(&self) -> String {
        format!("start_session/{}", self.unit_id)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StopSession {
    #[serde(skip)]
    pub unit_id: UnitId,
    pub booking_id: BookingId,
}

impl OsdTopic for StopSession {
    fn topic_name(&self) -> String {
        format!("stop_session/{}", self.unit_id)
    }
}
