#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing field: {0}")]
    MissingField(&'static str),
    #[error("Unit not found")]
    UnitNotFound,
    #[error("User not found")]
    UserNotFound,
    #[error("Booking not found")]
    BookingNotFound,
    #[error("Booking amendment not found")]
    BookingAmendmentNotFound,
    #[error("Invalid desired time range")]
    InvalidTimeRange,
    #[error("Specified time range is already occupied")]
    TimeRangeOccupied,
    #[error("Booking attachment is not available for the specified time range")]
    BookingAttachmentUnavailable,
    #[error("Error querying database: {0}")]
    Sqlx(#[from] sqlx::Error),
}
