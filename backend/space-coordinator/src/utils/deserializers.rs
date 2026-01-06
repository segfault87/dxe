use chrono::TimeDelta;
use serde::Deserialize;
use serde::de::Deserializer;

pub fn deserialize_time_delta_seconds<'de, D>(deserializer: D) -> Result<TimeDelta, D::Error>
where
    D: Deserializer<'de>,
{
    let value = i64::deserialize(deserializer)?;

    Ok(TimeDelta::seconds(value))
}

pub fn deserialize_time_delta_seconds_optional<'de, D>(
    deserializer: D,
) -> Result<Option<TimeDelta>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<i64>::deserialize(deserializer)?;

    Ok(value.map(TimeDelta::seconds))
}

pub fn deserialize_time_delta_milliseconds_optional<'de, D>(
    deserializer: D,
) -> Result<Option<TimeDelta>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<i64>::deserialize(deserializer)?;

    Ok(value.map(TimeDelta::milliseconds))
}
