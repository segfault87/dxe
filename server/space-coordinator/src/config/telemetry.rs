use std::path::PathBuf;

use dxe_types::TelemetryType;
use serde::Deserialize;

use crate::tasks::z2m_controller::DeviceName;

#[derive(Clone, Debug, Deserialize)]
pub struct Tasi653bConfig {
    pub serial_number: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundMeterDevice {
    Tasi653b(Tasi653bConfig),
}

#[derive(Clone, Debug, Deserialize)]
pub struct SoundMeterConfig {
    pub device: SoundMeterDevice,
}

#[derive(Debug, Deserialize)]
pub struct Z2mPowerMeterConfig {
    pub devices: Vec<DeviceName>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "class", rename_all = "snake_case")]
pub enum TableClass {
    SoundMeter(SoundMeterConfig),
    Z2mPowerMeter(Z2mPowerMeterConfig),
}

#[derive(Clone, Debug, Hash, Deserialize, Eq, PartialEq)]
pub struct TableKey(String);

impl std::fmt::Display for TableKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Deserialize)]
pub struct TableConfig {
    #[serde(flatten)]
    pub class: TableClass,
    pub name: TableKey,
    pub remote_type: Option<TelemetryType>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub output_path: PathBuf,
    pub tables: Vec<TableConfig>,
}
