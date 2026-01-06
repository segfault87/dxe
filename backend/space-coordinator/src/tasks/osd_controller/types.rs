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
    pub fuzzy: bool,
}
