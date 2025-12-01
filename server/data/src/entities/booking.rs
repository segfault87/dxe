use chrono::{DateTime, Utc};
use dxe_types::{AdhocReservationId, BookingId, IdentityId, UnitId};
use sqlx::FromRow;

use crate::entities::{Identity, User};

#[derive(Debug, Clone)]
pub struct OccupiedSlot {
    pub name: String,
    pub time_from: DateTime<Utc>,
    pub time_to: DateTime<Utc>,
    pub confirmed: bool,
}

#[derive(Debug, Clone)]
pub struct Booking {
    pub id: BookingId,
    pub unit_id: UnitId,
    pub holder: User,
    pub customer: Identity,
    pub time_from: DateTime<Utc>,
    pub time_to: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CashPaymentStatus {
    pub booking_id: BookingId,
    pub depositor_name: String,
    pub price: i64,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub refund_price: Option<i64>,
    pub refund_account: Option<String>,
    pub refunded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ItsokeyCredential {
    pub booking_id: BookingId,
    pub key: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct AdhocReservation {
    pub id: AdhocReservationId,
    pub unit_id: UnitId,
    pub holder: User,
    pub customer: Identity,
    pub time_from: DateTime<Utc>,
    pub time_to: DateTime<Utc>,
    pub temporary: bool,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct BookingChangeHistory {
    pub seq: i64,
    pub booking_id: BookingId,
    pub created_at: DateTime<Utc>,
    pub new_customer_id: Option<IdentityId>,
    pub new_time_from: Option<DateTime<Utc>>,
    pub new_time_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AudioRecording {
    pub booking_id: BookingId,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub expires_in: Option<DateTime<Utc>>,
}
