use dxe_extern::itsokey::{Error as ItsokeyError, ItsokeyClient};

use crate::config::{DoorLockBackend, DoorLockConfig, ItsokeyConfig};

#[derive(Clone, Debug)]
struct ItsokeyService {
    config: ItsokeyConfig,
    client: ItsokeyClient,
}

impl ItsokeyService {
    async fn open(&self) -> Result<(), ItsokeyError> {
        self.client.open(&self.config).await
    }
}

#[derive(Clone, Debug)]
enum Backend {
    Itsokey(ItsokeyService),
}

#[derive(Clone, Debug)]
pub struct DoorLockService {
    backend: Backend,
}

impl DoorLockService {
    pub fn new(config: DoorLockConfig) -> Self {
        let backend = match config.backend {
            DoorLockBackend::Itsokey => {
                if let Some(config) = config.itsokey {
                    Backend::Itsokey(ItsokeyService {
                        config,
                        client: ItsokeyClient::new(),
                    })
                } else {
                    panic!("Itsokey config is not provided.")
                }
            }
        };

        Self { backend }
    }

    pub async fn open(&self) -> Result<(), Error> {
        match &self.backend {
            Backend::Itsokey(service) => Ok(service.open().await?),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Itsokey(#[from] ItsokeyError),
}
