use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use parking_lot::Mutex;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::config::presence::{Config, PresenceIdentityConfig};
use crate::events::{Event, EventSender};
use crate::tables::{QualifiedPath, TablePublisher};
use crate::types::{Endpoint, EventId, PresenceEvent, PresenceRef, PublishKey, TenantId};

static PUBLISH_KEY_IS_PRESENT: PublishKey = PublishKey::new_const("is_present");

pub struct PresencePath;

impl QualifiedPath for PresencePath {
    type TableKey = PresenceRef;
    type Path = Endpoint;

    fn path(table_key: &Self::TableKey) -> Self::Path {
        Endpoint::Presence(table_key.clone())
    }
}

#[derive(Default)]
pub struct PresenceState {
    has_initialized: bool,
    pub is_present: bool,
    last_state: bool,
    last_seen_at: Option<DateTime<Utc>>,
}

pub struct PresenceMonitor {
    state: Arc<Mutex<HashMap<TenantId, PresenceState>>>,

    event_sender: EventSender,
    identities: HashMap<TenantId, PresenceIdentityConfig>,
    away_interval: TimeDelta,
    table: TablePublisher<PresenceRef, Endpoint, PresencePath>,
}

impl PresenceMonitor {
    pub async fn new(config: &Config, event_sender: EventSender) -> Self {
        let state = Arc::new(Mutex::new(Default::default()));

        let monitor = Self {
            state: state.clone(),

            event_sender,
            identities: config.identities.clone(),
            away_interval: config.away_interval,
            table: TablePublisher::new(),
        };

        monitor.ping().await;

        monitor
    }

    async fn ping(&self) {
        for (tenant_id, config) in self.identities.iter() {
            for address in config.scan_ips.iter() {
                let address = *address;
                let result = tokio::task::spawn_blocking(move || {
                    ping::new(address).timeout(Duration::from_secs(1)).send()
                })
                .await
                .unwrap();

                // For some reason it fails to decode ICMP packet and we are ignoring it anyways.
                if result.is_ok() || matches!(result, Err(ping::Error::DecodeV4Error)) {
                    let mut has_entered = false;

                    {
                        let mut states = self.state.lock();
                        let state = states.entry(tenant_id.clone()).or_default();
                        if !state.last_state && state.is_present {
                            log::info!(
                                "Presence of tenant {tenant_id} detected. endpoint: {address}"
                            );
                        } else if !state.is_present {
                            log::info!(
                                "Presence of tenant {tenant_id} state changed to true. endpoint: {address}"
                            );
                            state.is_present = true;
                            state.has_initialized = true;
                            has_entered = true;
                        }
                        state.last_state = true;
                        state.last_seen_at = Some(Utc::now());
                    }

                    if has_entered {
                        self.event_sender.publish(
                            EventId::Presence(tenant_id.clone(), PresenceEvent::Enter),
                            Event::Presence {
                                tenant_id: tenant_id.clone(),
                                r#type: PresenceEvent::Enter,
                            },
                        );
                        self.table.update_value(
                            PresenceRef::Tenant(tenant_id.clone()),
                            PUBLISH_KEY_IS_PRESENT.clone(),
                            serde_json::Value::Bool(true),
                        );
                    }

                    return;
                }
            }

            let mut has_left = false;
            {
                let mut states = self.state.lock();
                let state = states.entry(tenant_id.clone()).or_default();

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
            }

            if has_left {
                self.event_sender.publish(
                    EventId::Presence(tenant_id.clone(), PresenceEvent::Leave),
                    Event::Presence {
                        tenant_id: tenant_id.clone(),
                        r#type: PresenceEvent::Leave,
                    },
                );
                self.table.update_value(
                    PresenceRef::Tenant(tenant_id.clone()),
                    PUBLISH_KEY_IS_PRESENT.clone(),
                    serde_json::Value::Bool(false),
                );
            }
        }
    }

    pub fn task(self) -> Task {
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
