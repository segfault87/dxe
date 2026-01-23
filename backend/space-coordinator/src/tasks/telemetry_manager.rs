pub mod sound_meter;
pub mod z2m_aq;
pub mod z2m_power_meter;

use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::Display;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;

use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_s2s_shared::handlers::UploadTelemetryFileRequest;
use dxe_types::{BookingId, TelemetryType};
use futures::{FutureExt, StreamExt, select};
use parking_lot::Mutex;
use reqwest::multipart::{Form, Part};
use serde::Serialize;
use tokio::sync::broadcast;

use crate::callback::EventStateCallback;
use crate::client::DxeClient;
use crate::config::telemetry::{Config, Table, TableClass};
use crate::services::table_manager::TableManager;
use crate::services::telemetry::TelemetryService;
use crate::tables::TableUpdateReceiver;
use crate::tasks::telemetry_manager::sound_meter::SoundMeterTable;
use crate::tasks::telemetry_manager::z2m_aq::Z2mAirQualityTable;
use crate::tasks::telemetry_manager::z2m_power_meter::Z2mPowerMeterTable;
use crate::types::{Endpoint, PublishedValues};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct TableEntry {
    name: String,
    booking_id: BookingId,
}

impl TableEntry {
    pub fn path(&self) -> PathBuf {
        PathBuf::from(format!("{self}.csv"))
    }
}

impl Display for TableEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.name, self.booking_id)
    }
}

pub trait TableSpec {
    type State: Send + Sync;
    type Row: Serialize + Send + Sync + 'static;

    fn new_state(&self) -> Self::State;
    fn name(&self) -> String;
    fn endpoint(&self) -> Endpoint;
    fn remote_type(&self) -> Option<TelemetryType>;
    fn create_row(&self, state: &mut Self::State, values: PublishedValues) -> Option<Self::Row>;
}

pub struct TelemetryManager {
    service: TelemetryService<TableEntry>,
    client: DxeClient,
    table_manager: TableManager,

    table_update_handlers: Mutex<HashMap<String, tokio::task::JoinHandle<()>>>,
    event_state_handler: broadcast::Sender<(BookingId, bool)>,
}

impl TelemetryManager {
    pub fn new(
        config: &Config,
        client: DxeClient,
        table_manager: TableManager,
    ) -> Result<Arc<Self>, Box<dyn StdError>> {
        let service = TelemetryService::new(config.output_path.clone());

        let manager = Arc::new(Self {
            service,
            client,
            table_manager,

            table_update_handlers: Mutex::new(HashMap::new()),
            event_state_handler: broadcast::channel(10).0,
        });
        manager.clone().register_tables(config.tables.iter())?;

        Ok(manager)
    }

    async fn handle_incoming_message<S: TableSpec>(
        self: Arc<Self>,
        spec: &S,
        value: PublishedValues,
        states: &mut HashMap<TableEntry, S::State>,
    ) {
        for (key, state) in states.iter_mut() {
            if let Some(row) = spec.create_row(state, value.clone())
                && let Err(e) = self.service.write(key, row).await
            {
                log::warn!("Could not write CSV row: {e}");
            }
        }
    }

    async fn upload_telemetry_file(
        self: Arc<Self>,
        path: PathBuf,
        booking_id: BookingId,
        remote_type: TelemetryType,
    ) -> Result<(), Box<dyn StdError>> {
        let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();

        let form = Form::new()
            .part(
                "file",
                Part::file(path)
                    .await?
                    .file_name(file_name)
                    .mime_str("text/csv")?,
            )
            .part(
                "request",
                Part::text(serde_json::to_string(&UploadTelemetryFileRequest {
                    r#type: remote_type,
                })?)
                .mime_str("application/json;charset=UTF-8")?,
            );

        let _: serde_json::Value = self
            .client
            .post_multipart(&format!("/booking/{booking_id}/telemetry"), None, form)
            .await?;

        Ok(())
    }

    async fn table_handler_loop<S: TableSpec + Send + Sync + 'static>(
        self: Arc<Self>,
        spec: Arc<S>,
        mut rx: TableUpdateReceiver,
        mut event_state_rx: broadcast::Receiver<(BookingId, bool)>,
    ) {
        let mut states = HashMap::new();

        loop {
            select! {
                event_state = event_state_rx.recv().fuse() => {
                    match event_state {
                        Ok((booking_id, started)) => {
                            let table_entry = TableEntry {
                                name: spec.name(),
                                booking_id,
                            };

                            if started {
                                let path = table_entry.path();
                                match self.service.start(table_entry.clone(), path).await {
                                    Ok(()) => {
                                        log::info!("Telemetry {table_entry} started for {booking_id}...");
                                        states.insert(table_entry, spec.new_state());
                                    }
                                    Err(e) => {
                                        log::error!("Could not create telemetry log file for writing: {e}");
                                    }
                                }
                            } else {
                                match self.service.stop(&table_entry).await {
                                    Ok(path) => {
                                        if let Some(remote_type) = spec.remote_type() {
                                            log::info!("Telemetry {table_entry} finished. Uploading...");
                                            if let Err(e) = Arc::clone(&self).upload_telemetry_file(path, booking_id, remote_type).await {
                                                log::error!("Could not upload file to server: {e}");
                                            }
                                        } else {
                                            log::info!("Telemetry {table_entry} finished.");
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Could not stop telemetry logging: {e}");
                                    }
                                }
                                states.remove(&table_entry);
                            }
                         },
                         Err(e) => log::error!("Telemetry event stream stopped abruptly: {e}"),
                    }
                }
                value_result = rx.next().fuse() => {
                    if let Some(Ok(value)) = value_result {
                        Arc::clone(&self).handle_incoming_message::<S>(&spec, value, &mut states).await;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn register<S: TableSpec + Send + Sync + 'static>(self: Arc<Self>, spec: Arc<S>) {
        let table_name = spec.name();
        let rx = self.table_manager.subscribe(spec.endpoint());

        let event_state_rx = self.event_state_handler.subscribe();

        let arc_self = Arc::clone(&self);
        let task = tokio::task::spawn(async move {
            arc_self.table_handler_loop(spec, rx, event_state_rx).await;
        });

        self.table_update_handlers.lock().insert(table_name, task);
    }

    pub fn abort(&self) {
        for task in self.table_update_handlers.lock().values() {
            task.abort();
        }

        self.table_update_handlers.lock().clear();
    }

    fn register_tables<'a>(
        self: Arc<Self>,
        config: impl Iterator<Item = &'a Table>,
    ) -> Result<(), Box<dyn StdError>> {
        for table in config {
            match &table.class {
                TableClass::Z2mPowerMeter => {
                    let z2m_power_meter_table = Arc::new(Z2mPowerMeterTable::new(
                        table.name.clone(),
                        table.endpoint.clone(),
                        table.remote_type,
                    ));
                    self.clone().register(z2m_power_meter_table);
                }
                TableClass::Z2mAq => {
                    let z2m_aq_table = Arc::new(Z2mAirQualityTable::new(
                        table.name.clone(),
                        table.endpoint.clone(),
                        table.remote_type,
                    ));
                    self.clone().register(z2m_aq_table);
                }
                TableClass::SoundMeter => {
                    let sound_meter_table = Arc::new(SoundMeterTable::new(
                        table.name.clone(),
                        table.endpoint.clone(),
                        table.remote_type,
                    ));
                    self.clone().register(sound_meter_table);
                }
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl EventStateCallback<BookingWithUsers> for TelemetryManager {
    async fn on_event_start(
        self: Arc<Self>,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if !buffered {
            let _ = self.event_state_handler.send((event.booking.id, true));
        }

        Ok(())
    }

    async fn on_event_end(
        self: Arc<Self>,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if !buffered {
            let _ = self.event_state_handler.send((event.booking.id, false));
        }

        Ok(())
    }
}
