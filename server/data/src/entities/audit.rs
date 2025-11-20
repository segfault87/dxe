use chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::types::{BookingId, UserId};

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
