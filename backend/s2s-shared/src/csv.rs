use std::str::FromStr;

use num_traits::Num;
use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_optional_number<'de, D, V: FromStr + Num + Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<V>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value.parse().map_err(|_| {
            serde::de::Error::invalid_type(serde::de::Unexpected::Str(&value), &"a number")
        })?))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SoundMeterRow {
    #[serde(deserialize_with = "deserialize_optional_number")]
    pub decibel_level: Option<f64>,
    #[serde(deserialize_with = "deserialize_optional_number")]
    pub decibel_level_10: Option<i16>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Z2mPowerMeterRow {
    pub instantaneous_wattage: f64,
    pub power_usage_kwh: f64,
}
