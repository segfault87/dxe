use std::path::PathBuf;

use dxe_types::TelemetryType;
use serde::Deserialize;

use crate::types::Endpoint;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableClass {
    SoundMeter,
    Z2mPowerMeter,
    Z2mAq,
}

#[derive(Debug, Deserialize)]
pub struct Table {
    pub class: TableClass,
    pub endpoint: Endpoint,
    pub name: String,
    pub remote_type: Option<TelemetryType>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub output_path: PathBuf,
    pub tables: Vec<Table>,
}
