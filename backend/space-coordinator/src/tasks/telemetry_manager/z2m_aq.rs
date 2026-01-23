use chrono::{DateTime, TimeDelta, Utc};
use dxe_s2s_shared::csv::Z2mAirQualityRow;
use dxe_types::TelemetryType;

use crate::types::{Endpoint, PublishKey, PublishedValues};

pub const PUBLISH_RATE: TimeDelta = TimeDelta::seconds(10);
pub const KEY_CO2: PublishKey = PublishKey::new_const("co2");
pub const KEY_FORMALDEHYD: PublishKey = PublishKey::new_const("formaldehyd");
pub const KEY_HUMIDITY: PublishKey = PublishKey::new_const("humidity");
pub const KEY_TEMPERATURE: PublishKey = PublishKey::new_const("temperature");
pub const KEY_VOC: PublishKey = PublishKey::new_const("voc");

pub struct Z2mAirQualityState {
    current_co2: i64,
    current_formaldehyd: i64,
    current_humidity: f64,
    current_temperature: f64,
    current_voc: i64,
    last_published_at: DateTime<Utc>,
}

pub struct Z2mAirQualityTable {
    name: String,
    endpoint: Endpoint,
    remote_type: Option<TelemetryType>,
}

impl Z2mAirQualityTable {
    pub fn new(name: String, endpoint: Endpoint, remote_type: Option<TelemetryType>) -> Self {
        Self {
            name,
            endpoint,
            remote_type,
        }
    }
}

impl super::TableSpec for Z2mAirQualityTable {
    type State = Z2mAirQualityState;
    type Row = Z2mAirQualityRow;

    fn new_state(&self) -> Self::State {
        Z2mAirQualityState {
            current_co2: 0,
            current_formaldehyd: 0,
            current_humidity: 0.0,
            current_temperature: 0.0,
            current_voc: 0,
            last_published_at: DateTime::<Utc>::MIN_UTC,
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
        if let Some(co2) = values.get(&KEY_CO2)
            && let Some(co2) = co2.as_i64()
        {
            state.current_co2 = co2;
        }

        if let Some(formaldehyd) = values.get(&KEY_FORMALDEHYD)
            && let Some(formaldehyd) = formaldehyd.as_i64()
        {
            state.current_formaldehyd = formaldehyd;
        }

        if let Some(humidity) = values.get(&KEY_HUMIDITY)
            && let Some(humidity) = humidity.as_f64()
        {
            state.current_humidity = humidity;
        }

        if let Some(temperature) = values.get(&KEY_TEMPERATURE)
            && let Some(temperature) = temperature.as_f64()
        {
            state.current_temperature = temperature;
        }

        if let Some(voc) = values.get(&KEY_VOC)
            && let Some(voc) = voc.as_i64()
        {
            state.current_voc = voc;
        }

        let now = Utc::now();

        if now - state.last_published_at > PUBLISH_RATE {
            let row = Self::Row {
                co2: state.current_co2,
                formaldehyd: state.current_formaldehyd,
                humidity: state.current_humidity,
                temperature: state.current_temperature,
                voc: state.current_voc,
            };

            state.last_published_at = now;
            Some(row)
        } else {
            None
        }
    }
}
