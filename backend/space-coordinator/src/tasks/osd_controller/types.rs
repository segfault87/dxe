use chrono::{DateTime, Utc};
use dxe_types::BookingId;
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
#[serde(rename_all(serialize = "camelCase"))]
pub struct MixerChannelData {
    pub level: Option<i8>,
    pub pan: Option<i8>,
    pub reverb: Option<i8>,
    pub mute: Option<bool>,
    pub eq_high_level: Option<i8>,
    pub eq_high_freq: Option<i8>,
    pub eq_mid_level: Option<i8>,
    pub eq_mid_freq: Option<i8>,
    pub eq_mid_q: Option<i8>,
    pub eq_low_level: Option<i8>,
    pub eq_low_freq: Option<i8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MixerGlobalData {
    pub master_level: Option<i8>,
    pub monitor_level: Option<i8>,
}
