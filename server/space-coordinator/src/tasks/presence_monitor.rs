use std::sync::Arc;
use std::time::Duration;
use std::{net::IpAddr, sync::Mutex};

use chrono::{DateTime, TimeDelta, Utc};
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::callback::PresenceCallback;
use crate::config::PresenceMonitorConfig;

#[derive(Default)]
pub struct PresenceState {
    has_initialized: bool,
    pub is_present: bool,
    pub last_state: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
}

pub struct PresenceMonitor {
    state: Arc<Mutex<PresenceState>>,

    scan_ips: Vec<IpAddr>,
    away_interval: TimeDelta,
    callbacks: Vec<Arc<dyn PresenceCallback + Send + Sync + 'static>>,
}

impl PresenceMonitor {
    pub fn new(config: &PresenceMonitorConfig) -> (Arc<Mutex<PresenceState>>, Self) {
        let state = Arc::new(Mutex::new(Default::default()));

        (
            state.clone(),
            Self {
                state,

                scan_ips: config.scan_ips.clone(),
                away_interval: TimeDelta::seconds(config.away_interval_secs),
                callbacks: vec![],
            },
        )
    }

    pub fn add_callback<T>(&mut self, callback: Arc<T>)
    where
        T: PresenceCallback + Send + Sync + 'static,
    {
        self.callbacks.push(callback);
    }

    async fn ping(self: Arc<Self>) {
        for address in self.scan_ips.iter() {
            let address = *address;
            let result = tokio::task::spawn_blocking(move || {
                ping::new(address).timeout(Duration::from_secs(1)).send()
            })
            .await
            .unwrap();
            if result.is_ok() {
                let mut has_entered = false;

                {
                    let mut state = self.state.lock().unwrap();
                    if !state.last_state && state.is_present {
                        log::info!("Presence detected. endpoint: {address}");
                    } else if !state.is_present {
                        log::info!("Presence state changed to true. endpoint: {address}");
                        state.is_present = true;
                        state.has_initialized = true;
                        has_entered = true;
                    }
                    state.last_state = true;
                    state.last_seen_at = Some(Utc::now());
                }

                if has_entered {
                    for callback in self.callbacks.iter() {
                        if let Err(e) = callback.on_enter().await {
                            log::error!("Error while setting presence: {e}");
                        }
                    }
                }

                return;
            }
        }

        let mut has_left = false;
        {
            let mut state = self.state.lock().unwrap();

            if !state.has_initialized {
                state.has_initialized = true;
                has_left = true;
            }

            if let Some(last_seen_at) = state.last_seen_at {
                if state.last_state {
                    log::info!(
                        "Presence disappeared. It will take effect after {} seconds.",
                        self.away_interval.num_seconds()
                    );
                    state.last_state = false;
                }

                if Utc::now() - last_seen_at > self.away_interval && state.is_present {
                    log::info!("Presence state changed to false.");
                    state.is_present = false;
                    has_left = true;
                }
            }
        }

        if has_left {
            for callback in self.callbacks.iter() {
                if let Err(e) = callback.on_leave().await {
                    log::error!("Error while setting presence: {e}");
                }
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
}
