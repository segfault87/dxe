use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Arc;

use crate::callback::AlertCallback;
use crate::config::NotificationAlert;
use crate::services::notification::NotificationService;
use crate::types::AlertId;

pub struct NotificationPublisher {
    notification_service: NotificationService,

    alerts: HashMap<AlertId, NotificationAlert>,
}

impl NotificationPublisher {
    pub fn new<'a>(
        notification_service: NotificationService,
        alerts: impl Iterator<Item = &'a NotificationAlert>,
    ) -> Arc<Self> {
        Arc::new(Self {
            notification_service,

            alerts: alerts.map(|v| (v.alert_id.clone(), v.clone())).collect(),
        })
    }
}

#[async_trait::async_trait]
impl AlertCallback for NotificationPublisher {
    async fn on_alert(&self, alert_id: AlertId, started: bool) -> Result<(), Box<dyn StdError>> {
        if let Some(alert) = self.alerts.get(&alert_id)
            && started
        {
            Ok(self
                .notification_service
                .notify(alert.priority, alert.message.clone())
                .await?)
        } else {
            Ok(())
        }
    }
}
