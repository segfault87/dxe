pub mod entities;
pub mod handlers;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn to_utc(&self) -> DateTime<Utc> {
        self.0
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(value: DateTime<Utc>) -> Self {
        Self(value)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(value: Timestamp) -> Self {
        value.0
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.0.timestamp_millis())
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Timestamp;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a i64 integer")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Timestamp(DateTime::from_timestamp_millis(v as i64).ok_or(
                    de::Error::invalid_value(de::Unexpected::Unsigned(v), &self),
                )?))
            }
        }

        deserializer.deserialize_u64(Visitor)
    }
}
