pub mod sound_meter;
pub mod z2m_power_meter;

use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::fmt::Display;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;

use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_s2s_shared::handlers::UploadTelemetryFileRequest;
use dxe_types::{BookingId, TelemetryType};
use futures::{FutureExt, select};
use parking_lot::Mutex;
use reqwest::multipart::{Form, Part};
use serde::Serialize;
use tokio::sync::{broadcast, mpsc};

use crate::callback::EventStateCallback;
use crate::client::DxeClient;
use crate::config::telemetry::{Config, TableClass, TableConfig, TableKey};
use crate::services::telemetry::TelemetryService;
use crate::tasks::telemetry_manager::sound_meter::{SoundMeter, SoundMeterTable};
use crate::tasks::telemetry_manager::z2m_power_meter::Z2mPowerMeterTable;
use crate::tasks::z2m_controller::Z2mController;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct TableEntry {
    table_key: TableKey,
    booking_id: BookingId,
}

impl TableEntry {
    pub fn path(&self) -> PathBuf {
        PathBuf::from(format!("{self}.csv"))
    }
}

impl Display for TableEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.table_key, self.booking_id)
    }
}

pub trait TableSpec {
    type State: Send + Sync;
    type Value: Clone + Send + Sync;
    type Row: Serialize + Send + Sync + 'static;

    fn new_state(&self) -> Self::State;
    fn table_key(&self) -> TableKey;
    fn remote_type(&self) -> Option<TelemetryType>;
    fn create_row(&self, state: &mut Self::State, value: Self::Value) -> Self::Row;
}

pub struct TelemetryManager {
    service: TelemetryService<TableEntry>,
    client: DxeClient,

    table_update_handlers: Mutex<HashMap<TableKey, tokio::task::JoinHandle<()>>>,
    event_state_handler: broadcast::Sender<(BookingId, bool)>,
}

impl TelemetryManager {
    pub fn new(config: &Config, client: DxeClient) -> Arc<Self> {
        let service = TelemetryService::new(config.output_path.clone());

        Arc::new(Self {
            service,
            client,
            table_update_handlers: Mutex::new(HashMap::new()),
            event_state_handler: broadcast::channel(10).0,
        })
    }

    async fn handle_incoming_message<S: TableSpec>(
        self: Arc<Self>,
        spec: &S,
        value: S::Value,
        states: &mut HashMap<TableEntry, S::State>,
    ) {
        for (key, state) in states.iter_mut() {
            let row = spec.create_row(state, value.clone());

            if let Err(e) = self.service.write(key, row).await {
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
        mut rx: mpsc::UnboundedReceiver<S::Value>,
        mut event_state_rx: broadcast::Receiver<(BookingId, bool)>,
    ) {
        let mut states = HashMap::new();

        loop {
            select! {
                event_state = event_state_rx.recv().fuse() => {
                    match event_state {
                        Ok((booking_id, started)) => {
                            let table_entry = TableEntry {
                                table_key: spec.table_key(),
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
                value_result = rx.recv().fuse() => {
                    if let Some(value) = value_result {
                        Arc::clone(&self).handle_incoming_message::<S>(&spec, value, &mut states).await;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    pub fn register_spec<S: TableSpec + Send + Sync + 'static>(
        self: Arc<Self>,
        spec: Arc<S>,
    ) -> mpsc::UnboundedSender<S::Value> {
        let (tx, rx) = mpsc::unbounded_channel();

        let table_key = spec.clone().table_key();

        let event_state_rx = Arc::clone(&self).event_state_handler.subscribe();

        let arc_self = Arc::clone(&self);
        let task = tokio::task::spawn(async move {
            arc_self.table_handler_loop(spec, rx, event_state_rx).await;
        });

        self.table_update_handlers.lock().insert(table_key, task);

        tx
    }

    pub fn abort(&self) {
        for task in self.table_update_handlers.lock().values() {
            task.abort();
        }

        self.table_update_handlers.lock().clear();
    }

    pub fn register_tables_from_config(
        self: Arc<Self>,
        z2m_controller: &mut Z2mController,
        config: &Vec<TableConfig>,
    ) -> Result<Vec<tokio::task::JoinHandle<()>>, Box<dyn StdError>> {
        let mut tasks = vec![];

        for table in config {
            match &table.class {
                TableClass::Z2mPowerMeter(power_meter) => {
                    let z2m_power_meter_table = Arc::new(Z2mPowerMeterTable::new(
                        table.name.clone(),
                        table.remote_type,
                    ));
                    for device_name in power_meter.devices.iter() {
                        if let Some(device) = z2m_controller.get_device(device_name) {
                            z2m_power_meter_table.update_power_usage(
                                device_name.clone(),
                                z2m_controller.read_power_meter(device)?.sum_kwh,
                            );
                        }
                    }
                    let tx = Arc::clone(&self).register_spec(z2m_power_meter_table.clone());

                    z2m_controller.register_power_meter_telemetry_hook(
                        table.name.clone(),
                        HashSet::from_iter(power_meter.devices.clone().into_iter()),
                        tx,
                        z2m_power_meter_table,
                    );
                }
                TableClass::SoundMeter(sound_meter) => {
                    let sound_meter_table =
                        Arc::new(SoundMeterTable::new(table.name.clone(), table.remote_type));
                    let tx = Arc::clone(&self).register_spec(sound_meter_table);

                    let sound_meter = SoundMeter::new(sound_meter.clone(), tx);
                    tasks.push(sound_meter.start());
                }
            }
        }

        Ok(tasks)
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
