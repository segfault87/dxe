use dxe_extern::ntfy::{Channel, NtfyClient};

use crate::config::{NotificationBackend, NotificationConfig, NotificationPriority};

impl From<NotificationPriority> for Channel {
    fn from(value: NotificationPriority) -> Self {
        match value {
            NotificationPriority::Default => Channel::General,
            NotificationPriority::High => Channel::Important,
            NotificationPriority::Low => Channel::Minor,
        }
    }
}

#[derive(Clone, Debug)]
pub enum NotificationService {
    Ntfy(NtfyClient),
}

impl NotificationService {
    pub fn new(config: &NotificationConfig) -> Self {
        match &config.backend {
            NotificationBackend::Ntfy(config) => Self::Ntfy(NtfyClient::new(config)),
        }
    }

    pub async fn notify(
        &self,
        priority: NotificationPriority,
        message: String,
    ) -> Result<(), Error> {
        match &self {
            Self::Ntfy(client) => Ok(client.send(priority.into(), message).await?),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Ntfy(#[from] dxe_extern::ntfy::Error),
}
