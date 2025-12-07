use chrono::{DateTime, Utc};
use dxe_types::{
    AdhocParkingId, AdhocReservationId, BookingId, ForeignPaymentId, IdentityId, SpaceId,
    TelemetryType, UnitId, UserId,
};
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
pub struct TossPaymentStatus {
    pub id: ForeignPaymentId,
    pub user_id: UserId,
    pub temporary_reservation_id: AdhocReservationId,
    pub booking_id: Option<BookingId>,
    pub price: i64,
    pub payment_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub refund_price: Option<i64>,
    pub refunded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AdhocReservation {
    pub id: AdhocReservationId,
    pub unit_id: UnitId,
    pub holder: User,
    pub customer: Identity,
    pub time_from: DateTime<Utc>,
    pub time_to: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
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

#[derive(Debug, Clone, FromRow)]
pub struct TelemetryFile {
    pub booking_id: BookingId,
    pub r#type: TelemetryType,
    pub file_name: String,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AdhocParking {
    pub id: AdhocParkingId,
    pub space_id: SpaceId,
    pub time_from: DateTime<Utc>,
    pub time_to: DateTime<Utc>,
    pub license_plate_number: String,
    pub created_at: DateTime<Utc>,
}
