use chrono::{DateTime, Utc};
use dxe_types::{
    AdhocParkingId, AdhocReservationId, BookingAmendmentId, BookingId, SpaceId, TelemetryType,
    UnitId,
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
pub struct BookingAmendment {
    pub id: BookingAmendmentId,
    pub booking_id: BookingId,
    pub original_time_from: DateTime<Utc>,
    pub original_time_to: DateTime<Utc>,
    pub desired_time_from: DateTime<Utc>,
    pub desired_time_to: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum ProductDiscriminator {
    Booking,
    BookingAmendment,
}

#[derive(Debug, Clone)]
pub enum Product {
    Booking(Box<Booking>),
    Amendment(BookingAmendment),
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
