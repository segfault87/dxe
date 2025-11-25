use dxe_extern::amano::{AmanoClient, CarParkExemptionResult};

use crate::config::CarparkExemptionConfig;

pub enum CarparkExemptionService {
    Amano(AmanoClient),
}

impl CarparkExemptionService {
    pub fn new(config: &CarparkExemptionConfig) -> Self {
        match config.backend {
            crate::config::CarparkExemptionBackend::Amano => Self::Amano(AmanoClient::new(
                config.amano.as_ref().expect("amano config"),
            )),
        }
    }

    pub async fn exempt(&self, license_plate_number: &str) -> Result<(), Error> {
        match self {
            Self::Amano(client) => match client.exempt(license_plate_number).await? {
                CarParkExemptionResult::Success => {
                    log::info!("Parking exemption for {license_plate_number} applied successfully");
                    Ok(())
                }
                _ => Ok(()),
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Amano(#[from] dxe_extern::amano::Error),
}
