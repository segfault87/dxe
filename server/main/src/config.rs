use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, FixedOffset, TimeDelta, Utc};
use dxe_types::{SpaceId, UnitId};
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub url: url::Url,
}

#[derive(Clone, Deserialize, Debug)]
pub struct KakaoAuthConfig {
    pub client_id: String,
    pub auth_client_secret: String,
}

impl dxe_extern::kakao::KakaoRestApiConfig for KakaoAuthConfig {
    fn client_id(&self) -> &str {
        &self.client_id
    }

    fn auth_client_secret(&self) -> &str {
        &self.auth_client_secret
    }
}

#[derive(Deserialize, Debug)]
pub struct AuthConfig {
    pub kakao: KakaoAuthConfig,
}

#[derive(Deserialize, Debug)]
pub struct JwtConfig {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

impl JwtConfig {
    pub fn key_pair(&self) -> Result<ed25519_compact::KeyPair, ed25519_compact::Error> {
        Ok(ed25519_compact::KeyPair {
            pk: ed25519_compact::PublicKey::from_slice(self.public_key.as_slice())?,
            sk: ed25519_compact::SecretKey::from_slice(self.secret_key.as_slice())?,
        })
    }
}

#[serde_as]
#[derive(Clone, Deserialize, Debug)]
pub struct TimeZoneConfig {
    #[serde_as(as = "DisplayFromStr")]
    pub timezone: FixedOffset,
}

impl TimeZoneConfig {
    pub fn convert(&self, dt: DateTime<Utc>) -> DateTime<FixedOffset> {
        dt.with_timezone(&self.timezone)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct UnitBookingConfig {
    pub base_price: i64,
    pub price_per_hour: i64,
}

impl UnitBookingConfig {
    fn calculate_price(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> i64 {
        let delta = to - from;
        let hours = delta.num_hours();

        self.base_price + self.price_per_hour * hours
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct BookingConfig {
    pub lookahead_days: i64,
    pub max_booking_hours: i64,
    pub buffer_time: (TimeDelta, TimeDelta),
    pub refund_rates: Vec<(i64, i64)>,
    pub units: HashMap<UnitId, UnitBookingConfig>,
}

impl BookingConfig {
    pub fn sanitize(&mut self) {
        self.refund_rates.sort_by(|a, b| b.0.cmp(&a.0));
    }

    pub fn is_in_buffer(
        &self,
        now: &DateTime<Utc>,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> bool {
        let lower = from - self.buffer_time.0;
        let upper = to + self.buffer_time.1;

        now >= &lower && now < &upper
    }

    pub fn calculate_price(
        &self,
        unit_id: &UnitId,
        time_from: DateTime<Utc>,
        time_to: DateTime<Utc>,
    ) -> Result<i64, ()> {
        let Some(unit_booking_config) = self.units.get(unit_id) else {
            return Err(());
        };

        Ok(unit_booking_config.calculate_price(time_from, time_to))
    }

    pub fn calculate_refund_price(
        &self,
        timezone_config: &TimeZoneConfig,
        booking_price: i64,
        booking_date: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<i64, ()> {
        let delta = booking_date - now;

        if delta.num_hours() < 0 {
            return Err(());
        }

        if let Some(imminent_refund_rate) = self
            .refund_rates
            .iter()
            .find(|(hour, _)| hour == &0)
            .map(|(_, v)| *v)
        {
            let local_booking_date = timezone_config.convert(booking_date).date_naive();
            let local_today = timezone_config.convert(now).date_naive();

            if local_booking_date == local_today {
                return Ok(booking_price * imminent_refund_rate / 100);
            }
        }

        let mut desired_rate = None;
        for (hours, rate) in self.refund_rates.iter() {
            if hours == &0 {
                continue;
            } else if delta.num_hours() <= *hours {
                desired_rate = Some(*rate);
            }
        }

        Ok(if let Some(desired_rate) = desired_rate {
            booking_price * desired_rate / 100
        } else {
            booking_price
        })
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DoorLockBackend {
    Itsokey,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ItsokeyConfig {
    device_id: String,
    dp_id: String,
    password: String,
}

impl dxe_extern::itsokey::ItsokeyConfig for ItsokeyConfig {
    fn device_id(&self) -> &str {
        self.device_id.as_str()
    }

    fn dp_id(&self) -> &str {
        self.dp_id.as_str()
    }

    fn password(&self) -> &str {
        self.password.as_str()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct DoorLockConfig {
    pub backend: DoorLockBackend,
    pub itsokey: Option<ItsokeyConfig>,
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

#[derive(Clone, Deserialize, Debug)]
pub struct BiztalkConfig {
    bs_id: String,
    password: String,
    sender_key: String,
}

impl dxe_extern::biztalk::BiztalkConfig for BiztalkConfig {
    fn bs_id(&self) -> &str {
        &self.bs_id
    }

    fn password(&self) -> &str {
        &self.password
    }

    fn sender_key(&self) -> &str {
        &self.sender_key
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct MessagingConfig {
    pub biztalk: Option<BiztalkConfig>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UrlConfig {
    pub base_url: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SpaceSecurityConfig {
    pub public_key: Vec<u8>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SpaceConfig {
    pub doorlock: Option<DoorLockConfig>,
    pub security: SpaceSecurityConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GoogleCalendarConfig {
    pub calendar_id: String,
    pub identity: String,
}

impl dxe_extern::google_cloud::calendar::GoogleCalendarConfig for GoogleCalendarConfig {
    fn calendar_id(&self) -> &str {
        &self.calendar_id
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct GoogleApiConfig {
    pub service_account_path: PathBuf,
    pub calendar: GoogleCalendarConfig,
}

impl dxe_extern::google_cloud::GoogleCloudAuthConfig for GoogleApiConfig {
    fn service_account_path(&self) -> &PathBuf {
        &self.service_account_path
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(flatten)]
    pub url: UrlConfig,
    pub aes_key: Vec<u8>,
    #[serde(flatten)]
    pub timezone: TimeZoneConfig,
    pub booking: BookingConfig,
    pub jwt: JwtConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub spaces: HashMap<SpaceId, SpaceConfig>,
    pub notifications: NotificationConfig,
    pub messaging: MessagingConfig,
    pub google_apis: Option<GoogleApiConfig>,
}
