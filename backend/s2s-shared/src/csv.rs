use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SoundMeterRow {
    pub decibel_level: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Z2mPowerMeterRow {
    pub instantaneous_wattage: f64,
    pub power_usage_kwh: f64,
}
