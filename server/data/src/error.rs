#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing field: {0}")]
    MissingField(&'static str),
    #[error("Unit not found")]
    UnitNotFound,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid desired time range")]
    InvalidTimeRange,
    #[error("Specified time range is already occupied")]
    TimeRangeOccupied,
    #[error("Error querying database: {0}")]
    Sqlx(#[from] sqlx::Error),
}
