pub mod z2m;

use std::net::IpAddr;

use dxe_types::SpaceId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AmanoConfig {
    pub url_base: url::Url,
    pub lot_id: String,
    pub user_id: String,
    pub hashed_password: String,
}

impl dxe_extern::amano::AmanoConfig for AmanoConfig {
    fn url_base(&self) -> &url::Url {
        &self.url_base
    }

    fn lot_id(&self) -> &str {
        &self.lot_id
    }

    fn user_id(&self) -> &str {
        &self.user_id
    }

    fn hashed_password(&self) -> &str {
        &self.hashed_password
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CarparkExemptionBackend {
    Amano,
}

#[derive(Debug, Deserialize)]
pub struct CarparkExemptionConfig {
    pub backend: CarparkExemptionBackend,
    pub amano: Option<AmanoConfig>,
}

#[derive(Debug, Deserialize)]
pub struct PresenceMonitorConfig {
    pub scan_ips: Vec<IpAddr>,
    pub away_interval_secs: i64,
}

#[derive(Debug, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub space_id: SpaceId,
    pub url_base: url::Url,
    pub request_expires_in_secs: i64,
    pub private_key: Vec<u8>,
    pub mqtt: MqttConfig,
    pub carpark_exemption: Option<CarparkExemptionConfig>,
    pub presence_monitor: PresenceMonitorConfig,
    pub z2m: z2m::Config,
}

impl Config {
    pub fn request_expires_in(&self) -> chrono::TimeDelta {
        chrono::TimeDelta::seconds(self.request_expires_in_secs)
    }
}
