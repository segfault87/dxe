use chrono::{DateTime, Utc};
use dxe_types::BookingId;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlertSeverity {
    Urgent,
    Normal,
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
}
