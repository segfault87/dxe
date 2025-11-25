use dxe_types::{BookingId, IdentityId, UnitId, UserId};
use serde::{Deserialize, Serialize};

use crate::Timestamp;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Booking {
    pub id: BookingId,
    pub unit_id: UnitId,
    pub date_start_w_buffer: Timestamp,
    pub date_start: Timestamp,
    pub date_end: Timestamp,
    pub date_end_w_buffer: Timestamp,
    pub customer_id: IdentityId,
    pub customer_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub license_plate_number: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct BookingWithUsers {
    pub booking: Booking,
    pub users: Vec<User>,
}
