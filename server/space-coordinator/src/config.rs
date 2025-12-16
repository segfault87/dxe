pub mod telemetry;
pub mod z2m;

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;

use dxe_types::{SpaceId, UnitId};
use serde::Deserialize;

use crate::services::mqtt::MqttTopicPrefix;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AlertPriority {
    High,
    #[default]
    Default,
    Low,
}

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
    pub endpoint_name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum NotificationBackend {
    Ntfy,
}

#[derive(Clone, Deserialize, Debug)]
pub struct NtfyConfig {
    token: Option<String>,
    channels: HashMap<String, String>,
}

impl dxe_extern::ntfy::NtfyConfig for NtfyConfig {
    fn access_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    fn channel(&self, channel: dxe_extern::ntfy::Channel) -> &str {
        match channel {
            dxe_extern::ntfy::Channel::General => self.channels.get("general").unwrap(),
            dxe_extern::ntfy::Channel::Important => self.channels.get("important").unwrap(),
            dxe_extern::ntfy::Channel::Minor => self.channels.get("minor").unwrap(),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct NotificationConfig {
    pub backend: NotificationBackend,
    pub ntfy: Option<NtfyConfig>,
}

#[derive(Deserialize, Debug)]
pub struct GoogleDriveConfig {
    pub parent: String,
}

impl dxe_extern::google_cloud::drive::GoogleDriveConfig for GoogleDriveConfig {
    fn parent(&self) -> &str {
        &self.parent
    }
}

#[derive(Deserialize, Debug)]
pub struct GoogleApiConfig {
    pub service_account_path: PathBuf,
    pub drive: GoogleDriveConfig,
}

impl dxe_extern::google_cloud::GoogleCloudAuthConfig for GoogleApiConfig {
    fn service_account_path(&self) -> &PathBuf {
        &self.service_account_path
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct AudioRecorderConfig {
    pub pw_record_bin: PathBuf,
    pub lame_bin: PathBuf,
    pub target_device: String,
    pub mp3_bitrate: i32,
    pub sampling_rate: i32,
    pub path_prefix: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OsdConfig {
    pub topic_prefix: MqttTopicPrefix,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub space_id: SpaceId,
    pub url_base: url::Url,
    pub request_expires_in_secs: i64,
    pub private_key: Vec<u8>,
    pub mqtt: MqttConfig,
    pub notifications: NotificationConfig,
    pub carpark_exemption: Option<CarparkExemptionConfig>,
    pub presence_monitor: PresenceMonitorConfig,
    pub google_apis: GoogleApiConfig,
    pub audio_recorder: HashMap<UnitId, AudioRecorderConfig>,
    pub z2m: z2m::Config,
    pub osd: OsdConfig,
    pub telemetry: telemetry::Config,
}

impl Config {
    pub fn request_expires_in(&self) -> chrono::TimeDelta {
        chrono::TimeDelta::seconds(self.request_expires_in_secs)
    }
}
