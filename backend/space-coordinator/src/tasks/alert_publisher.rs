use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use futures::StreamExt;
use futures::stream::select_all;
use parking_lot::Mutex;
use tokio::task::JoinHandle;

use crate::config::events::AlertConfig;
use crate::events::{Event, EventSender};
use crate::services::table_manager::{TableManager, TableSnapshot};
use crate::types::{AlertId, EventId};

pub struct AlertPublisher {
    alerts: HashMap<AlertId, AlertConfig>,

    sender: EventSender,
    tasks: Mutex<HashMap<AlertId, JoinHandle<()>>>,
}

impl Drop for AlertPublisher {
    fn drop(&mut self) {
        for (_, task) in self.tasks.lock().drain() {
            task.abort();
        }
    }
}

impl AlertPublisher {
    pub fn new(configs: &HashMap<AlertId, AlertConfig>, sender: EventSender) -> Self {
        Self {
            alerts: configs.clone(),
            sender,
            tasks: Mutex::new(HashMap::new()),
        }
    }

    async fn alert_monitor_loop(
        self: Arc<Self>,
        alert_id: AlertId,
        alert: AlertConfig,
        table_manager: TableManager,
    ) {
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

                let now = Utc::now();
                if let Some(debounce) = alert.debounce
                    && now - last_alert_at < debounce
                {
                    continue;
                }

                let result = match alert.condition.test(&snapshot) {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("Could not test alert condition {alert_id}: {e}");
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
                                log::info!("alert {alert_id} will be started after grace...");

                                grace_time = Some(now);
                                continue;
                            }
                        } else {
                            last_alert_at = now;
                        }

                        alert_fired = true;

                        log::info!("Firing alert {}", alert_id);

                        self.sender.publish(
                            EventId::Alert(alert_id.clone()),
                            Event::Alert {
                                grace: alert.grace,
                                debounce: alert.debounce,
                                unit_ids: alert.unit_ids.clone(),
                            },
                        );
                    }
                } else if alert_fired {
                    alert_fired = false;
                } else if grace_time.is_some() {
                    grace_time = None;

                    log::info!("Cancelling alert {alert_id}");
                }
            }
        }
    }

    pub fn start(self, table_manager: TableManager) -> Arc<Self> {
        let arc_self = Arc::new(self);

        for (id, alert) in arc_self.alerts.clone().into_iter() {
            let cloned_id = id.clone();
            let cloned_arc_self = arc_self.clone();
            let cloned_table_manager = table_manager.clone();
            let task = tokio::task::spawn(async move {
                cloned_arc_self
                    .clone()
                    .alert_monitor_loop(cloned_id, alert, cloned_table_manager)
                    .await;
            });
            arc_self.tasks.lock().insert(id, task);
        }

        arc_self
    }
}
