use std::collections::HashMap;
use std::sync::Arc;

use futures::StreamExt;
use parking_lot::Mutex;
use tokio::task::JoinHandle;

use crate::config::NotificationAlert;
use crate::events::EventSender;
use crate::services::notification::NotificationService;
use crate::types::EventId;

pub struct NotificationPublisher {
    notification_service: NotificationService,

    alerts: HashMap<EventId, Vec<NotificationAlert>>,
    event_receiver: Mutex<Option<JoinHandle<()>>>,
}

impl NotificationPublisher {
    pub fn new<'a>(
        notification_service: NotificationService,
        configs: impl Iterator<Item = &'a NotificationAlert>,
    ) -> Self {
        let mut alerts: HashMap<EventId, Vec<NotificationAlert>> = HashMap::new();

        for config in configs {
            for event_id in config.event_ids.iter() {
                alerts
                    .entry(event_id.clone())
                    .or_default()
                    .push(config.clone());
            }
        }

        Self {
            notification_service,

            alerts,
            event_receiver: Mutex::new(None),
        }
    }

    pub fn start(self, event_sender: EventSender) -> Arc<Self> {
        let arc_self = Arc::new(self);

        let mut receiver = event_sender.subscribe(arc_self.clone().alerts.keys().cloned());
        let cloned_arc_self = arc_self.clone();
        let task = tokio::task::spawn(async move {
            while let Some(item) = receiver.next().await {
                if let Ok((event_id, _)) = item
                    && let Some(alerts) = cloned_arc_self.clone().alerts.get(&event_id)
                {
                    for alert in alerts.iter() {
                        if let Err(e) = cloned_arc_self
                            .clone()
                            .notification_service
                            .notify(alert.priority, alert.message.clone())
                            .await
                        {
                            log::error!("Could not send notification for event {event_id}: {e}");
                        }
                    }
                }
            }
        });
        *arc_self.clone().event_receiver.lock() = Some(task);

        arc_self
    }
}
