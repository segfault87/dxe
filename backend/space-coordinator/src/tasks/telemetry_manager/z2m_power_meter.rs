use chrono::{DateTime, TimeDelta, Utc};
use dxe_s2s_shared::csv::Z2mPowerMeterRow;
use dxe_types::TelemetryType;

use crate::types::{Endpoint, PublishKey, PublishedValues};

pub const PUBLISH_DURATION: TimeDelta = TimeDelta::seconds(10);
pub const KEY_ENERGY: PublishKey = PublishKey::new_const("energy");
pub const KEY_POWER: PublishKey = PublishKey::new_const("power");

pub struct State {
    initial_energy_usage_kwh: Option<f64>,
    last_published_at: DateTime<Utc>,
    current_wattage: f64,
    current_energy_usage_kwh: f64,
}

pub struct Z2mPowerMeterTable {
    name: String,
    endpoint: Endpoint,
    remote_type: Option<TelemetryType>,
}

impl Z2mPowerMeterTable {
    pub fn new(name: String, endpoint: Endpoint, remote_type: Option<TelemetryType>) -> Self {
        Self {
            name,
            endpoint,
            remote_type,
        }
    }
}

impl super::TableSpec for Z2mPowerMeterTable {
    type State = State;
    type Row = Z2mPowerMeterRow;

    fn new_state(&self) -> Self::State {
        Self::State {
            initial_energy_usage_kwh: None,
            last_published_at: DateTime::<Utc>::MIN_UTC,
            current_energy_usage_kwh: 0.0,
            current_wattage: 0.0,
        }
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn endpoint(&self) -> Endpoint {
        self.endpoint.clone()
    }

    fn remote_type(&self) -> Option<TelemetryType> {
        self.remote_type
    }

    fn create_row(&self, state: &mut Self::State, values: PublishedValues) -> Option<Self::Row> {
        if let Some(energy) = values.get(&KEY_ENERGY)
            && let Some(energy) = energy.as_f64()
        {
            let initial = if let Some(initial) = state.initial_energy_usage_kwh {
                initial
            } else {
                state.initial_energy_usage_kwh = Some(energy);
                energy
            };

            state.current_energy_usage_kwh = energy - initial;
        }

        if let Some(power) = values.get(&KEY_POWER)
            && let Some(power) = power.as_f64()
        {
            state.current_wattage = state.current_wattage.max(power);
        }

        let now = Utc::now();

        if now - state.last_published_at > PUBLISH_DURATION {
            let row = Self::Row {
                instantaneous_wattage: state.current_wattage,
                power_usage_kwh: state.current_energy_usage_kwh,
            };

            state.current_wattage = 0.0;
            state.last_published_at = now;
            Some(row)
        } else {
            None
        }
    }
}
