#[derive(Copy, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "sql", derive(sqlx::Type), sqlx(transparent))]
pub struct UserId(i64);

impl From<i64> for UserId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "sql", derive(sqlx::Type), sqlx(transparent))]
pub struct GroupId(i64);

impl From<i64> for GroupId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Identity {
    User(UserId),
    Group(GroupId),
}
