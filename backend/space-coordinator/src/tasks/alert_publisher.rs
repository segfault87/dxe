use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_types::{BookingId, UnitId};
use futures::StreamExt;
use futures::stream::select_all;
use parking_lot::Mutex;
use tokio::task::JoinHandle;

use crate::callback::{AlertCallback, EventStateCallback, PresenceCallback};
use crate::config::alert::Alert;
use crate::services::table_manager::{TableManager, TableSnapshot};
use crate::types::AlertId;

pub struct AlertPublisher {
    alerts: HashMap<AlertId, Alert>,
    presence: Mutex<bool>,
    active_units: Mutex<HashMap<UnitId, HashSet<BookingId>>>,

    tasks: Mutex<HashMap<AlertId, JoinHandle<()>>>,
    callbacks: Vec<Arc<dyn AlertCallback + Send + Sync + 'static>>,
}

impl AlertPublisher {
    pub fn new<'a>(configs: impl Iterator<Item = &'a Alert>, is_present: bool) -> Self {
        let alerts = configs
            .map(|v| (v.id.clone(), v.clone()))
            .collect::<HashMap<_, _>>();

        Self {
            alerts,
            presence: Mutex::new(is_present),
            active_units: Mutex::new(HashMap::new()),

            tasks: Mutex::new(HashMap::new()),
            callbacks: Vec::new(),
        }
    }

    pub fn add_callback<T>(&mut self, callback: Arc<T>)
    where
        T: AlertCallback + Send + Sync + 'static,
    {
        self.callbacks.push(callback);
    }

    async fn alert_monitor_loop(self: Arc<Self>, alert: Alert, table_manager: TableManager) {
        let keys = alert.condition.keys();
        let endpoints = keys
            .iter()
            .map(|v| v.endpoint.clone())
            .collect::<HashSet<_>>();

        let receivers = endpoints
            .into_iter()
            .map(|endpoint| {
                table_manager
                    .subscribe(endpoint.clone())
                    .map(move |item| item.map(|v| (endpoint.clone(), v)))
            })
            .collect::<Vec<_>>();

        let mut snapshot = TableSnapshot::new();

        let mut alert_fired = false;
        let mut last_alert_at = DateTime::<Utc>::MIN_UTC;
        let mut grace_time: Option<DateTime<Utc>> = None;

        let mut combined_receivers = select_all(receivers);
        while let Some(item) = combined_receivers.next().await {
            if let Ok((endpoint, table)) = item {
                snapshot.update(&endpoint, table);

                if let Some(presence) = alert.presence
                    && *self.presence.lock() != presence
                {
                    continue;
                }

                if let Some(unit_ids) = &alert.bookings
                    && unit_ids
                        .iter()
                        .filter(|id| self.clone().is_unit_active(id))
                        .count()
                        == 0
                {
                    continue;
                }

                let now = Utc::now();
                if let Some(snooze) = alert.snooze
                    && now - last_alert_at < snooze
                {
                    continue;
                }

                let result = match alert.condition.test(&snapshot) {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("Could not test alert condition {}: {e}", alert.id);
                        continue;
                    }
                };

                if result {
                    if !alert_fired {
                        if let Some(grace_period) = alert.grace {
                            if let Some(time) = grace_time {
                                if now - time > grace_period {
                                    grace_time = None;
                                    last_alert_at = now;
                                } else {
                                    continue;
                                }
                            } else {
                                log::info!("alert {} will be started after grace...", alert.id);

                                grace_time = Some(now);
                                continue;
                            }
                        } else {
                            last_alert_at = now;
                        }

                        alert_fired = true;

                        log::info!("Firing alert {}", alert.id);

                        for callback in self.callbacks.iter() {
                            if let Err(e) = callback.on_alert(alert.id.clone(), true).await {
                                log::warn!("Could not invoke alert callback for {}: {e}", alert.id);
                            }
                        }
                    }
                } else if alert_fired {
                    alert_fired = false;

                    for callback in self.callbacks.iter() {
                        if let Err(e) = callback.on_alert(alert.id.clone(), false).await {
                            log::warn!("Could not invoke alert callback for {}: {e}", alert.id);
                        }
                    }
                } else if grace_time.is_some() {
                    grace_time = None;

                    log::info!("Cancelling alert {}", alert.id);
                }
            }
        }
    }

    pub fn start(self, table_manager: TableManager) -> Arc<Self> {
        let arc_self = Arc::new(self);

        for alert in arc_self.clone().alerts.values() {
            let task = tokio::task::spawn(
                arc_self
                    .clone()
                    .alert_monitor_loop(alert.clone(), table_manager.clone()),
            );

            arc_self.clone().tasks.lock().insert(alert.id.clone(), task);
        }

        arc_self
    }

    pub fn is_unit_active(self: Arc<Self>, unit_id: &UnitId) -> bool {
        self.active_units
            .lock()
            .get(unit_id)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }
}

#[async_trait::async_trait]
impl EventStateCallback<BookingWithUsers> for AlertPublisher {
    async fn on_event_start(
        self: Arc<Self>,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if !buffered {
            self.active_units
                .lock()
                .entry(event.booking.unit_id.clone())
                .or_default()
                .insert(event.booking.id);
        }

        Ok(())
    }

    async fn on_event_end(
        self: Arc<Self>,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn StdError>> {
        if !buffered {
            self.active_units
                .lock()
                .get_mut(&event.booking.unit_id)
                .map(|v| v.remove(&event.booking.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl PresenceCallback for AlertPublisher {
    async fn on_enter(&self) -> Result<(), Box<dyn StdError>> {
        *self.presence.lock() = true;

        Ok(())
    }

    async fn on_leave(&self) -> Result<(), Box<dyn StdError>> {
        *self.presence.lock() = false;

        Ok(())
    }
}
