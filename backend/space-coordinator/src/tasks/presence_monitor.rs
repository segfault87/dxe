use std::collections::HashMap;
use std::iter::once;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use parking_lot::Mutex;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::config::presence::{Config, PresenceIdentityConfig};
use crate::events::{Event, EventSender};
use crate::services::influxdb::MetricValue;
use crate::tables::{QualifiedPath, TablePublisher};
use crate::tasks::metrics_exporter::{IntoDataPoint, MetricsExporterHandle};
use crate::types::{Endpoint, EventId, PresenceEvent, PresenceRef, PublishKey, TenantId};

static PUBLISH_KEY_IS_PRESENT: PublishKey = PublishKey::new_const("is_present");
static PUBLISH_KEY_COUNT: PublishKey = PublishKey::new_const("count");

pub struct PresencePath;

impl QualifiedPath for PresencePath {
    type TableKey = PresenceRef;
    type Path = Endpoint;

    fn path(table_key: &Self::TableKey) -> Self::Path {
        Endpoint::Presence(table_key.clone())
    }
}

#[derive(Clone)]
pub struct PresenceState {
    tenant_id: TenantId,
    has_initialized: bool,
    pub is_present: bool,
    last_state: bool,
    last_seen_at: Option<DateTime<Utc>>,
}

impl PresenceState {
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            has_initialized: false,
            is_present: false,
            last_state: false,
            last_seen_at: None,
        }
    }
}

impl IntoDataPoint for PresenceState {
    fn measurement() -> &'static str {
        "presence"
    }

    fn tags(&self) -> impl Iterator<Item = (&'static str, String)> {
        once(("tenant_id", self.tenant_id.to_string()))
    }

    fn fields(&self) -> impl Iterator<Item = (&'static str, MetricValue)> {
        once(("presence", self.is_present.into()))
    }
}

pub struct PresenceMonitor {
    state: Arc<Mutex<HashMap<TenantId, PresenceState>>>,

    event_sender: EventSender,
    metrics_exporter_handle: Option<MetricsExporterHandle>,

    identities: HashMap<TenantId, PresenceIdentityConfig>,
    away_interval: TimeDelta,
    table: TablePublisher<PresenceRef, Endpoint, PresencePath>,
    tenant_count: AtomicUsize,
}

impl PresenceMonitor {
    pub async fn new(
        config: &Config,
        event_sender: EventSender,
        metrics_exporter_handle: Option<MetricsExporterHandle>,
    ) -> Self {
        let state = Arc::new(Mutex::new(Default::default()));

        let monitor = Self {
            state: state.clone(),

            event_sender,
            metrics_exporter_handle,
            identities: config.identities.clone(),
            away_interval: config.away_interval,
            table: TablePublisher::new(),
            tenant_count: AtomicUsize::new(0),
        };

        monitor.ping().await;

        monitor
    }

    async fn ping(&self) {
        for (tenant_id, config) in self.identities.iter() {
            let mut found = false;

            for address in config.scan_ips.iter() {
                let address = *address;
                let result = tokio::task::spawn_blocking(move || {
                    ping::new(address).timeout(Duration::from_secs(1)).send()
                })
                .await
                .unwrap();

                // For some reason it fails to decode ICMP packet and we are ignoring it anyways.
                if result.is_ok() || matches!(result, Err(ping::Error::DecodeV4Error)) {
                    found = true;
                    break;
                }
            }

            let mut states = self.state.lock();
            let state = states
                .entry(tenant_id.clone())
                .or_insert_with(|| PresenceState::new(tenant_id.clone()));

            if found {
                let mut has_entered = false;

                if !state.last_state && state.is_present {
                    log::info!("Presence of tenant {tenant_id} detected.");
                } else if !state.is_present {
                    log::info!("Presence of tenant {tenant_id} state changed to true.");
                    state.is_present = true;
                    state.has_initialized = true;
                    has_entered = true;
                }
                state.last_state = true;
                state.last_seen_at = Some(Utc::now());

                if has_entered {
                    self.event_sender.publish(
                        EventId::Presence(tenant_id.clone(), PresenceEvent::Enter),
                        Event::Presence {
                            tenant_id: tenant_id.clone(),
                            r#type: PresenceEvent::Enter,
                        },
                    );
                    if let Some(metrics_exporter_handle) = self.metrics_exporter_handle.clone() {
                        metrics_exporter_handle.export(state);
                    }
                    self.table.update_value(
                        PresenceRef::Tenant(tenant_id.clone()),
                        PUBLISH_KEY_IS_PRESENT.clone(),
                        serde_json::Value::Bool(true),
                    );
                    self.tenant_count
                        .update(Ordering::Release, Ordering::Acquire, |v| v + 1);
                }
            } else {
                let mut has_left = false;
                if !state.has_initialized {
                    state.has_initialized = true;
                    has_left = true;
                }

                if let Some(last_seen_at) = state.last_seen_at {
                    if state.last_state {
                        log::info!(
                            "Tenant {tenant_id} disappeared. It will take effect after {} seconds.",
                            self.away_interval.num_seconds()
                        );
                        state.last_state = false;
                    }

                    if Utc::now() - last_seen_at > self.away_interval && state.is_present {
                        log::info!("Tenant {tenant_id} state changed to false.");
                        state.is_present = false;
                        has_left = true;
                    }
                }

                if has_left {
                    self.event_sender.publish(
                        EventId::Presence(tenant_id.clone(), PresenceEvent::Leave),
                        Event::Presence {
                            tenant_id: tenant_id.clone(),
                            r#type: PresenceEvent::Leave,
                        },
                    );
                    if let Some(metrics_exporter_handle) = self.metrics_exporter_handle.clone() {
                        metrics_exporter_handle.export(state);
                    }
                    self.table.update_value(
                        PresenceRef::Tenant(tenant_id.clone()),
                        PUBLISH_KEY_IS_PRESENT.clone(),
                        serde_json::Value::Bool(false),
                    );
                    if let Some(metrics_exporter_handle) = self.metrics_exporter_handle.clone() {
                        metrics_exporter_handle.export(state);
                    }
                    self.tenant_count
                        .update(Ordering::Release, Ordering::Acquire, |v| {
                            if v > 0 { v - 1 } else { 0 }
                        });
                }
            }
        }

        let tenants = self.tenant_count.load(Ordering::Relaxed);
        self.table.update_value(
            PresenceRef::Global,
            PUBLISH_KEY_COUNT.clone(),
            serde_json::Value::Number(serde_json::Number::from(tenants)),
        );
    }

    pub fn task(self) -> Task {
        // Initialize the table
        for tenant_id in self.identities.keys() {
            self.table.update_value(
                PresenceRef::Tenant(tenant_id.clone()),
                PUBLISH_KEY_IS_PRESENT.clone(),
                serde_json::Value::Bool(false),
            );
        }

        let arc_self = Arc::new(self);

        TaskBuilder::new("presence_monitor", move || {
            let arc_self = arc_self.clone();
            tokio::task::spawn(async move {
                arc_self.ping().await;
            });

            Ok(())
        })
        .every_seconds(30)
        .build()
    }

    pub fn publisher(&self) -> TablePublisher<PresenceRef, Endpoint, PresencePath> {
        self.table.clone()
    }
}
