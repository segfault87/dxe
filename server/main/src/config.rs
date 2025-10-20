use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DoorLockConfig {
    Hejhome { key: String },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CarparkExemptionConfig {
    Amano {
        host: Url,
        id: String,
        password: String,
        exempted_hours: i32,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MessagingConfig {
    Biztalk { api_key: String },
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    pub carpark_exemption: Option<CarparkExemptionConfig>,
    pub doorlock: Option<DoorLockConfig>,
}
