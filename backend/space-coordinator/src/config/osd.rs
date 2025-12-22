use serde::Deserialize;

use crate::tasks::osd_controller::types::AlertData;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct AlertConfig {
    pub sign_off_mins: i64,
    pub on_sign_in: Option<AlertData>,
    pub on_sign_off: Option<AlertData>,
}

impl AlertConfig {
    pub fn sign_off_duration(&self) -> chrono::TimeDelta {
        chrono::TimeDelta::minutes(self.sign_off_mins)
    }
}
