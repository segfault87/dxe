use std::convert::Infallible;
use std::fmt::Display;
use std::str::FromStr;
use std::{collections::HashMap, hash::Hash};

use dxe_types::UnitId;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

pub type PublishedValues = HashMap<PublishKey, serde_json::Value>;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(transparent)]
pub struct AlertId(String);

impl From<String> for AlertId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for AlertId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Eq)]
pub enum PublishKey {
    String(String),
    Const(&'static str),
}

impl<'de> Deserialize<'de> for PublishKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = PublishKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "A key to published value")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.to_owned().into())
            }
        }

        deserializer.deserialize_string(Visitor)
    }
}

impl Serialize for PublishKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl PublishKey {
    pub const fn new_const(value: &'static str) -> Self {
        Self::Const(value)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::String(value) => value.as_str(),
            Self::Const(value) => value,
        }
    }
}

impl From<String> for PublishKey {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl PartialEq for PublishKey {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Hash for PublishKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl Display for PublishKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => value.fmt(f),
            Self::Const(value) => value.fmt(f),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(transparent)]
pub struct DeviceId(String);

impl From<String> for DeviceId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, DeserializeFromStr, SerializeDisplay)]
pub enum DeviceType {
    Z2m,
    SoundMeter,
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z2m => write!(f, "z2m"),
            Self::SoundMeter => write!(f, "sound_meter"),
        }
    }
}

impl FromStr for DeviceType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "z2m" => Ok(Self::Z2m),
            "sound_meter" => Ok(Self::SoundMeter),
            other => Err(ParseError::DeviceType(other.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct DeviceRef {
    pub r#type: DeviceType,
    pub id: DeviceId,
}

impl Display for DeviceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.r#type, self.id)
    }
}

impl FromStr for DeviceRef {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (r#type, id) = s
            .split_once(':')
            .ok_or(ParseError::DeviceRef(s.to_owned()))?;

        Ok(Self {
            r#type: DeviceType::from_str(r#type)?,
            id: id.to_owned().into(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(transparent)]
pub struct MetricId(String);

impl From<String> for MetricId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for MetricId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, DeserializeFromStr, SerializeDisplay, Eq, PartialEq, Hash)]
pub enum PresenceRef {
    Global,
    Tenant(TenantId),
}

impl FromStr for PresenceRef {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self::Global)
        } else {
            Ok(Self::Tenant(s.to_owned().into()))
        }
    }
}

impl Display for PresenceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global => write!(f, ""),
            Self::Tenant(id) => write!(f, "{id}"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, DeserializeFromStr, SerializeDisplay)]
pub enum Endpoint {
    Metric(MetricId),
    Device(DeviceRef),
    Presence(PresenceRef),
    Bookings(UnitId),
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metric(id) => write!(f, "metric:{id}"),
            Self::Device(device) => write!(f, "device:{device}"),
            Self::Presence(presence) => write!(f, "presence:{presence}"),
            Self::Bookings(id) => write!(f, "bookings:{id}"),
        }
    }
}

impl FromStr for Endpoint {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, value) = s.split_once(':').unwrap_or((s, ""));

        match key {
            "metric" => Ok(Self::Metric(value.to_owned().into())),
            "device" => Ok(Self::Device(DeviceRef::from_str(value)?)),
            "presence" => Ok(Self::Presence(PresenceRef::from_str(value).unwrap())),
            "bookings" => Ok(Self::Bookings(value.to_owned().into())),
            other => Err(ParseError::Endpoint(other.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct EndpointKey {
    pub endpoint: Endpoint,
    pub key: PublishKey,
}

impl Display for EndpointKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.endpoint, self.key)
    }
}

impl FromStr for EndpointKey {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (value, key) = s
            .rsplit_once(':')
            .ok_or(ParseError::EndpointRef(s.to_owned()))?;

        Ok(EndpointKey {
            endpoint: Endpoint::from_str(value)?,
            key: key.to_owned().into(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Z2mDeviceId(String);

impl From<String> for Z2mDeviceId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl TryFrom<DeviceRef> for Z2mDeviceId {
    type Error = ParseError;

    fn try_from(value: DeviceRef) -> Result<Self, Self::Error> {
        match value.r#type {
            DeviceType::Z2m => Ok(Self(value.id.0.clone())),
            _ => Err(ParseError::NotZ2mDeviceId),
        }
    }
}

impl From<Z2mDeviceId> for DeviceId {
    fn from(value: Z2mDeviceId) -> Self {
        value.0.into()
    }
}

impl Display for Z2mDeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(transparent)]
pub struct TenantId(String);

impl From<String> for TenantId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct BookingEventId(String);

impl From<String> for BookingEventId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for BookingEventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceEvent {
    Enter,
    Leave,
}

impl FromStr for PresenceEvent {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "enter" => Ok(Self::Enter),
            "leave" => Ok(Self::Leave),
            _ => Err(ParseError::InvalidPresenceEvent(s.to_string())),
        }
    }
}

impl Display for PresenceEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Enter => write!(f, "enter"),
            Self::Leave => write!(f, "leave"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, DeserializeFromStr, SerializeDisplay)]
pub enum EventId {
    Booking(BookingEventId),
    Presence(TenantId, PresenceEvent),
    Alert(AlertId),
}

impl FromStr for EventId {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((prefix, remaining)) = s.split_once(":") {
            match prefix {
                "bookings" => Ok(EventId::Booking(remaining.to_owned().into())),
                "presence" => {
                    if let Some((tenant_id, event)) = remaining.split_once(":") {
                        Ok(EventId::Presence(
                            tenant_id.to_owned().into(),
                            PresenceEvent::from_str(event)?,
                        ))
                    } else {
                        Err(ParseError::EventPresenceType(remaining.to_owned()))
                    }
                }
                "alerts" => Ok(EventId::Alert(remaining.to_owned().into())),
                _ => Err(ParseError::EventType(s.to_owned())),
            }
        } else {
            Err(ParseError::EventType(s.to_owned()))
        }
    }
}

impl Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Booking(id) => write!(f, "bookings:{}", id),
            Self::Presence(tenant_id, event) => write!(f, "presence:{}:{}", tenant_id, event),
            Self::Alert(id) => write!(f, "alerts:{}", id),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid device type: {0}")]
    DeviceType(String),
    #[error("Invalid device reference value: {0}")]
    DeviceRef(String),
    #[error("Invalid endpoint value: {0}")]
    Endpoint(String),
    #[error("Invalid endpoint reference value: {0}")]
    EndpointRef(String),
    #[error("Invalid presence event: {0}")]
    InvalidPresenceEvent(String),
    #[error("Invalid event type: {0}")]
    EventType(String),
    #[error("Invalid presence event type: {0}")]
    EventPresenceType(String),
    #[error("Expecting Z2M device reference")]
    NotZ2mDeviceId,
}
