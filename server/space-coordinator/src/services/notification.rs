use dxe_extern::ntfy::{Channel, NtfyClient};

use crate::config::{AlertPriority, NotificationBackend, NotificationConfig};

impl From<AlertPriority> for Channel {
    fn from(value: AlertPriority) -> Self {
        match value {
            AlertPriority::Default => Channel::General,
            AlertPriority::High => Channel::Important,
            AlertPriority::Low => Channel::Minor,
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
            NotificationBackend::Ntfy => {
                Self::Ntfy(NtfyClient::new(config.ntfy.as_ref().expect("ntfy config")))
            }
        }
    }

    pub async fn notify(&self, priority: AlertPriority, message: String) -> Result<(), Error> {
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
