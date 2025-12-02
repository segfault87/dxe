use std::time::Duration;

use dxe_types::TelemetryType;
use futures::{FutureExt, StreamExt, select};
use serde::Serialize;
use tasi_sound_level_meter::TasiSoundLevelMeter;
use tokio::sync::mpsc;

use crate::config::telemetry::{SoundMeterConfig, SoundMeterDevice, TableKey};

const POLL_INTERVAL: Duration = Duration::from_secs(10);

pub struct State;

#[derive(Clone, Debug, Serialize)]
pub struct SoundMeterRow {
    decibel_level_10: i16,
}

pub struct SoundMeter {
    config: SoundMeterConfig,
    tx: mpsc::UnboundedSender<SoundMeterRow>,
}

impl SoundMeter {
    pub fn new(config: SoundMeterConfig, tx: mpsc::UnboundedSender<SoundMeterRow>) -> Self {
        Self { config, tx }
    }

    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::task::spawn(async move {
            let mut subsequent = false;
            loop {
                if !subsequent {
                    subsequent = true;
                } else {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }

                match &self.config.device {
                    SoundMeterDevice::Tasi653b(device) => {
                        let mut meter = match TasiSoundLevelMeter::new(device.serial_number.clone())
                        {
                            Ok(v) => v,
                            Err(e) => {
                                log::error!("Cannot open TASI sound meter: {e}");
                                continue;
                            }
                        };

                        let mut current_level = 0;

                        let mut interval = tokio::time::interval(POLL_INTERVAL);

                        loop {
                            select! {
                                _ = interval.tick().fuse() => {
                                    if current_level > 0 {
                                        let _ = self.tx.send(SoundMeterRow {
                                            decibel_level_10: current_level,
                                        });
                                        current_level = 0;
                                    }
                                }
                                meter = meter.next().fuse() => {
                                    if let Some(level) = meter {
                                        if level > current_level {
                                            current_level = level;
                                        }
                                    } else {
                                        log::warn!("Sound meter disconnected. retrying...");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                };
            }
        })
    }
}

pub struct SoundMeterTable {
    table_key: TableKey,
    remote_type: Option<TelemetryType>,
}

impl SoundMeterTable {
    pub fn new(table_key: TableKey, remote_type: Option<TelemetryType>) -> Self {
        Self {
            table_key,
            remote_type,
        }
    }
}

impl super::TableSpec for SoundMeterTable {
    type State = State;
    type Value = SoundMeterRow;
    type Row = SoundMeterRow;

    fn new_state(&self) -> Self::State {
        State
    }

    fn table_key(&self) -> TableKey {
        self.table_key.clone()
    }

    fn remote_type(&self) -> Option<TelemetryType> {
        self.remote_type
    }

    fn create_row(&self, _state: &mut Self::State, value: Self::Value) -> Self::Row {
        value
    }
}
