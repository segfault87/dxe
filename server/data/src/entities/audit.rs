use chrono::NaiveDateTime;
use dxe_types::{BookingId, UserId};
use sqlx::FromRow;

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct Activity {
    pub id: i64,
    pub user_id: UserId,
    pub booking_id: Option<BookingId>,
    pub event_name: String,
    pub level: String,
    pub created_at: NaiveDateTime,
    pub payload: serde_json::Value,
}
