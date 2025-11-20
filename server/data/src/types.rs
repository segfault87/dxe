use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Copy, Clone, Eq, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum IdentityProvider {
    Kakao,
    Administrator,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct UnitId(String);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct BookingId(Uuid);

impl BookingId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Display for BookingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct IdentityId(Uuid);

impl IdentityId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct UserId(Uuid);

impl From<IdentityId> for UserId {
    fn from(value: IdentityId) -> Self {
        Self(value.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct GroupId(Uuid);

impl From<IdentityId> for GroupId {
    fn from(value: IdentityId) -> Self {
        Self(value.0)
    }
}

impl std::fmt::Display for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct ReservationId(i64);

impl From<i64> for ReservationId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}
