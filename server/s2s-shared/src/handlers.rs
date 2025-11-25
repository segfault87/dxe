use std::collections::HashMap;

use dxe_types::UnitId;
use serde::{Deserialize, Serialize};

use crate::Timestamp;
use crate::entities::BookingWithUsers;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetPendingBookingsResponse {
    pub range_start: Timestamp,
    pub range_end: Timestamp,
    pub bookings: HashMap<UnitId, Vec<BookingWithUsers>>,
}
