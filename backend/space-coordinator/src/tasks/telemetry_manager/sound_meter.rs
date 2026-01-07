use chrono::{DateTime, TimeDelta, Utc};
use dxe_s2s_shared::csv::SoundMeterRow;
use dxe_types::TelemetryType;

use crate::types::{Endpoint, PublishKey, PublishedValues};

const PUBLISH_RATE: TimeDelta = TimeDelta::seconds(10);
const KEY_SOUND_LEVEL: PublishKey = PublishKey::new_const("sound_level");

pub struct State {
    last_published_at: DateTime<Utc>,
    max_decibel_level: f64,
}

pub struct SoundMeterTable {
    name: String,
    endpoint: Endpoint,
    remote_type: Option<TelemetryType>,
}

impl SoundMeterTable {
    pub fn new(name: String, endpoint: Endpoint, remote_type: Option<TelemetryType>) -> Self {
        Self {
            name,
            endpoint,
            remote_type,
        }
    }
}

impl super::TableSpec for SoundMeterTable {
    type State = State;
    type Row = SoundMeterRow;

    fn new_state(&self) -> Self::State {
        State {
            last_published_at: DateTime::<Utc>::MIN_UTC,
            max_decibel_level: 0.0,
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
        let now = Utc::now();

        if let Some(value) = values.get(&KEY_SOUND_LEVEL).and_then(|v| v.as_f64()) {
            if now - state.last_published_at < PUBLISH_RATE {
                state.max_decibel_level = state.max_decibel_level.max(value);
                None
            } else {
                state.last_published_at = now;
                let decibel_level = state.max_decibel_level.max(value);
                state.max_decibel_level = 0.0;
                Some(SoundMeterRow {
                    decibel_level: Some(decibel_level),
                    decibel_level_10: None,
                })
            }
        } else {
            None
        }
    }
}
