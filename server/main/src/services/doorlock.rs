use std::collections::HashMap;

use dxe_extern::itsokey::{Error as ItsokeyError, ItsokeyClient};
use dxe_types::SpaceId;

use crate::config::{DoorLockBackend, ItsokeyConfig, SpaceConfig};

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
    backends: HashMap<SpaceId, Backend>,
}

impl DoorLockService {
    pub fn new(config: &HashMap<SpaceId, SpaceConfig>) -> Self {
        let mut backends = HashMap::new();

        for (space_id, space_config) in config.iter() {
            if let Some(doorlock) = &space_config.doorlock {
                let backend = match doorlock.backend {
                    DoorLockBackend::Itsokey => {
                        if let Some(config) = &doorlock.itsokey {
                            Backend::Itsokey(ItsokeyService {
                                config: config.clone(),
                                client: ItsokeyClient::new(),
                            })
                        } else {
                            panic!("Itsokey config is not provided.")
                        }
                    }
                };
                backends.insert(space_id.clone(), backend);
            }
        }

        Self { backends }
    }

    pub async fn open(&self, space_id: &SpaceId) -> Result<(), Error> {
        let Some(backend) = self.backends.get(space_id) else {
            return Err(Error::NoConfiguration(space_id.clone()));
        };

        match backend {
            Backend::Itsokey(service) => Ok(service.open().await?),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Itsokey(#[from] ItsokeyError),
    #[error("No doorlock configuration found for space {0}")]
    NoConfiguration(SpaceId),
}
