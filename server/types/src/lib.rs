use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(rename_all = "lowercase"))]
pub enum IdentityProvider {
    Kakao,
    Handle,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct UnitId(String);

impl From<String> for UnitId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for UnitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct SpaceId(String);

impl From<String> for SpaceId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for SpaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct IdentityId(Uuid);

impl IdentityId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct UserId(Uuid);

impl From<IdentityId> for UserId {
    fn from(value: IdentityId) -> Self {
        Self(value.0)
    }
}

impl From<UserId> for IdentityId {
    fn from(value: UserId) -> Self {
        Self(value.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct GroupId(Uuid);

impl From<IdentityId> for GroupId {
    fn from(value: IdentityId) -> Self {
        Self(value.0)
    }
}

impl From<GroupId> for IdentityId {
    fn from(value: GroupId) -> Self {
        Self(value.0)
    }
}

impl std::fmt::Display for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct AdhocReservationId(i64);

impl From<i64> for AdhocReservationId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for AdhocReservationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(rename_all = "snake_case"))]
#[serde(rename_all = "snake_case")]
pub enum TelemetryType {
    PowerUsageTotal,
    PowerUsageRoom,
    SoundMeter,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct AdhocParkingId(i64);

impl From<i64> for AdhocParkingId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for AdhocParkingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct ForeignPaymentId(Uuid);

impl From<Uuid> for ForeignPaymentId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl Display for ForeignPaymentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ForeignPaymentId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}
