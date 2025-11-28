use std::collections::HashMap;
use std::fmt::Display;

use dxe_types::UnitId;
use serde::{Deserialize, Serialize};

use crate::Timestamp;
use crate::entities::{BookingWithUsers, Unit};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BookingType {
    All,
    Pending,
    Confirmed,
}

impl Display for BookingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Pending => write!(f, "pending"),
            Self::Confirmed => write!(f, "confirmed"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetBookingsQuery {
    pub r#type: BookingType,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetBookingsResponse {
    pub range_start: Timestamp,
    pub range_end: Timestamp,
    pub bookings: HashMap<UnitId, Vec<BookingWithUsers>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetUnitsResponse {
    pub units: Vec<Unit>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateAudioRequest {
    pub url: url::Url,
    pub expires_in: Timestamp,
}
