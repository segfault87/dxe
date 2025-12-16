use dxe_extern::ntfy::NtfyClient;
use tokio::sync::mpsc;

use crate::config::{NotificationBackend, NotificationConfig, NtfyConfig};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Priority {
    High,
    Default,
    Low,
}

#[derive(Clone, Debug, Default)]
struct NoopService;

impl NoopService {
    pub fn notify(&self, priority: Priority, message: String) -> Result<(), Error> {
        let level = match priority {
            Priority::High => "high",
            Priority::Default => "default",
            Priority::Low => "low",
        };
        log::info!("New notification ({level}): {message}");

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct NtfyService {
    client: NtfyClient,
}

impl NtfyService {
    pub fn new(config: NtfyConfig) -> Self {
        Self {
            client: NtfyClient::new(&config),
        }
    }

    pub async fn notify(&self, priority: Priority, message: String) -> Result<(), Error> {
        let channel = match priority {
            Priority::High => dxe_extern::ntfy::Channel::Important,
            Priority::Default => dxe_extern::ntfy::Channel::General,
            Priority::Low => dxe_extern::ntfy::Channel::Minor,
        };

        self.client.send(channel, message).await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
enum Backend {
    Noop(NoopService),
    Ntfy(NtfyService),
}

pub struct NotificationService {
    backend: Backend,
}

impl NotificationService {
    async fn notify(&self, priority: Priority, message: String) -> Result<(), Error> {
        match &self.backend {
            Backend::Noop(service) => service.notify(priority, message),
            Backend::Ntfy(service) => service.notify(priority, message).await,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NotificationSender {
    tx: mpsc::UnboundedSender<(Priority, String)>,
}

impl NotificationSender {
    pub fn enqueue(&self, priority: Priority, message: String) {
        self.tx.send((priority, message)).unwrap();
    }
}

impl NotificationService {
    pub fn new(config: NotificationConfig) -> Self {
        let backend = match config.backend {
            NotificationBackend::Noop => Backend::Noop(Default::default()),
            NotificationBackend::Ntfy => Backend::Ntfy(NtfyService::new(config.ntfy.unwrap())),
        };

        Self { backend }
    }
}

pub fn spawn_notification_service_task(
    config: NotificationConfig,
) -> (tokio::task::JoinHandle<()>, NotificationSender) {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let handle = tokio::task::spawn(async move {
        let service = NotificationService::new(config);

        while let Some((priority, message)) = rx.recv().await {
            if let Err(e) = service.notify(priority, message).await {
                log::warn!("Cannot send notification: {e}");
            }
        }
    });

    (handle, NotificationSender { tx })
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Ntfy(#[from] dxe_extern::ntfy::Error),
}
