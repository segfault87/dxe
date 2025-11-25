use std::sync::Arc;
use std::time::Duration;
use std::{net::IpAddr, sync::Mutex};

use chrono::{DateTime, TimeDelta, Utc};
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::config::PresenceMonitorConfig;

#[derive(Default)]
pub struct PresenceState {
    pub is_present: bool,
    pub last_state: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
}

pub struct PresenceMonitor {
    state: Arc<Mutex<PresenceState>>,

    scan_ips: Vec<IpAddr>,
    away_interval: TimeDelta,
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
            },
        )
    }

    fn ping(&self) {
        for address in self.scan_ips.iter() {
            let result = ping::new(*address).timeout(Duration::from_secs(1)).send();
            if result.is_ok() {
                let mut state = self.state.lock().unwrap();
                if !state.last_state && state.is_present {
                    log::info!("Presence detected. endpoint: {address}");
                } else if !state.is_present {
                    log::info!("Presence state changed to true. endpoint: {address}");
                    state.is_present = true;
                }
                state.last_state = true;
                state.last_seen_at = Some(Utc::now());
                return;
            }
        }

        let mut state = self.state.lock().unwrap();

        let Some(last_seen_at) = state.last_seen_at else {
            return;
        };

        if state.last_state {
            log::info!(
                "Presence disappeared. It will take effect after {} seconds.",
                self.away_interval.num_seconds()
            );
            state.last_state = false;
        }

        if Utc::now() - last_seen_at > self.away_interval {
            log::info!("Presence state changed to false.");
            state.is_present = false;
        }
    }

    pub fn task(self) -> Task {
        let arc_self = Arc::new(self);

        TaskBuilder::new("presence_monitor", move || {
            let arc_self = arc_self.clone();
            std::thread::spawn(move || {
                arc_self.ping();
            });

            Ok(())
        })
        .every_seconds(30)
        .build()
    }
}
