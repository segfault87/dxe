mod converters;

use chrono::{DateTime, FixedOffset};
use dxe_types::{AdhocReservationId, BookingId, GroupId, UnitId, UserId};
use serde::Serialize;

pub use converters::IntoView;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfUser {
    pub id: UserId,
    pub name: String,
    pub license_plate_number: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub is_administrator: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub owner_id: UserId,
    pub is_open: bool,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupWithUsers {
    pub id: GroupId,
    pub name: String,
    pub owner_id: UserId,
    pub is_open: bool,
    pub created_at: DateTime<FixedOffset>,
    pub users: Vec<User>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Identity {
    User(User),
    Group(Group),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BookingStatus {
    Pending,
    Confirmed,
    Overdue,
    Canceled,
    Buffered,
    InProgress,
    Complete,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Booking {
    pub id: BookingId,
    pub unit_id: UnitId,
    pub holder: User,
    pub customer: Identity,
    pub booking_start: DateTime<FixedOffset>,
    pub booking_end: DateTime<FixedOffset>,
    pub booking_hours: i64,
    pub created_at: DateTime<FixedOffset>,
    pub confirmed_at: Option<DateTime<FixedOffset>>,
    pub is_confirmed: bool,
    pub canceled_at: Option<DateTime<FixedOffset>>,
    pub is_canceled: bool,
    pub status: BookingStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingCashPaymentStatus {
    pub depositor_name: String,
    pub price: i64,
    pub confirmed_at: Option<DateTime<FixedOffset>>,
    pub refund_price: Option<i64>,
    pub refund_account: Option<String>,
    pub refunded_at: Option<DateTime<FixedOffset>>,
    pub is_refund_requested: bool,
    pub is_refunded: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdhocReservation {
    pub id: AdhocReservationId,
    pub holder: User,
    pub customer: Identity,
    pub reservation_start: DateTime<FixedOffset>,
    pub reservation_end: DateTime<FixedOffset>,
    pub reserved_hours: i64,
    pub temporary: bool,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OccupiedSlot {
    pub masked_name: String,
    pub booking_date: DateTime<FixedOffset>,
    pub booking_hours: i64,
    pub confirmed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingWithPayments {
    pub booking: Booking,
    pub payment: Option<BookingCashPaymentStatus>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioRecording {
    pub booking_id: BookingId,
    pub url: url::Url,
    pub created_at: DateTime<FixedOffset>,
    pub expires_in: Option<DateTime<FixedOffset>>,
}
