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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Z2mAirQualityRow {
    pub co2: i64,
    pub formaldehyd: i64,
    pub humidity: f64,
    pub temperature: f64,
    pub voc: i64,
}
