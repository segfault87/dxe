use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SoundMeterRow {
    pub decibel_level: Option<f64>,
    pub decibel_level_10: Option<i16>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Z2mPowerMeterRow {
    pub instantaneous_wattage: f64,
    pub power_usage_kwh: f64,
}
