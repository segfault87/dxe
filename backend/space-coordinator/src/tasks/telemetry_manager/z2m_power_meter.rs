use std::collections::HashMap;
use std::time::Duration;

use dxe_s2s_shared::csv::Z2mPowerMeterRow;
use dxe_types::TelemetryType;
use parking_lot::Mutex;

use crate::config::telemetry::TableKey;
use crate::tasks::z2m_controller::DeviceName;

pub const PUBLISH_DURATION: Duration = Duration::from_secs(10);

pub struct State {
    initial_power_usage_kwh: f64,
}

pub struct Z2mPowerMeterTable {
    table_key: TableKey,
    remote_type: Option<TelemetryType>,
    current_power_usage_kwh: Mutex<HashMap<DeviceName, f64>>,
}

impl Z2mPowerMeterTable {
    pub fn new(name: TableKey, remote_type: Option<TelemetryType>) -> Self {
        Self {
            table_key: name,
            remote_type,
            current_power_usage_kwh: Default::default(),
        }
    }

    pub fn update_power_usage(&self, device_name: DeviceName, usage: f64) {
        self.current_power_usage_kwh
            .lock()
            .insert(device_name.clone(), usage);
    }
}

impl super::TableSpec for Z2mPowerMeterTable {
    type State = State;
    type Value = Z2mPowerMeterRow;
    type Row = Z2mPowerMeterRow;

    fn new_state(&self) -> Self::State {
        Self::State {
            initial_power_usage_kwh: self.current_power_usage_kwh.lock().values().sum(),
        }
    }

    fn table_key(&self) -> TableKey {
        self.table_key.clone()
    }

    fn remote_type(&self) -> Option<TelemetryType> {
        self.remote_type
    }

    fn create_row(&self, state: &mut Self::State, value: Self::Value) -> Self::Row {
        Self::Row {
            instantaneous_wattage: value.instantaneous_wattage,
            power_usage_kwh: value.power_usage_kwh - state.initial_power_usage_kwh,
        }
    }
}
