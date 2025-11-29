use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

pub const SCOPE: &str = "https://www.googleapis.com/auth/calendar.events";

const CALENDAR_EVENTS_URL: &str =
    "https://www.googleapis.com/calendar/v3/calendars/{calendar_id}/events";
const CALENDAR_EVENT_URL: &str =
    "https://www.googleapis.com/calendar/v3/calendars/{calendar_id}/events/{event_id}";

pub trait GoogleCalendarConfig {
    fn calendar_id(&self) -> &str;
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EventVisibility {
    Default,
    Public,
    Private,
    Confidential,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ExtendedProperties {
    pub private: HashMap<String, String>,
    pub shared: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DateTimeRepresentation {
    pub date_time: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: EventId,
    pub summary: String,
    pub description: String,
    pub start: DateTimeRepresentation,
    pub end: DateTimeRepresentation,
    pub visibility: EventVisibility,
    pub extended_properties: ExtendedProperties,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct EventId(String);

impl From<String> for EventId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct GoogleCalendarClient {
    credential: super::CredentialManager,
    client: reqwest::Client,

    calendar_id: String,
}

impl GoogleCalendarClient {
    pub fn new(credential: super::CredentialManager, config: &impl GoogleCalendarConfig) -> Self {
        Self {
            credential,
            client: reqwest::Client::new(),

            calendar_id: config.calendar_id().to_owned(),
        }
    }

    pub async fn create_event(&self, event: Event) -> Result<(), Error> {
        let token = self.credential.get_token(&[SCOPE]).await?;

        let url = CALENDAR_EVENTS_URL.replace("{calendar_id}", &self.calendar_id);

        let response = self
            .client
            .post(url)
            .bearer_auth(token.as_str())
            .json(&event)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::Calendar(response.json().await?))
        }
    }

    pub async fn delete_event(&self, event_id: &EventId) -> Result<(), Error> {
        let token = self.credential.get_token(&[SCOPE]).await?;

        let url = CALENDAR_EVENT_URL
            .replace("{calendar_id}", &self.calendar_id)
            .replace("{event_id}", &event_id.to_string());

        let response = self
            .client
            .delete(url)
            .bearer_auth(token.as_str())
            .send()
            .await?;

        if response.status().is_client_error() {
            Err(Error::Calendar(response.json().await?))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("GCP authentication error: {0}")]
    Gcp(#[from] super::Error),
    #[error("Calendar API error: {0}")]
    Calendar(serde_json::Value),
}
