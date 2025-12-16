use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use parking_lot::Mutex;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Publish, QoS};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::config::MqttConfig;

#[derive(Clone, Debug, Eq)]
pub enum MqttTopicPrefix {
    Const(&'static str),
    String(String),
}

pub const EMPTY_TOPIC_PREFIX: MqttTopicPrefix = MqttTopicPrefix::new_const("");

impl MqttTopicPrefix {
    pub const fn new_const(prefix: &'static str) -> Self {
        Self::Const(prefix)
    }

    pub fn new(prefix: String) -> Self {
        Self::String(prefix)
    }

    pub fn from_topic_name(topic_name: &str) -> Option<Self> {
        if let Some((prefix, _)) = topic_name.split_once('/') {
            Some(Self::String(prefix.to_owned()))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Const(v) => v,
            Self::String(v) => v.as_str(),
        }
    }

    pub fn topic(&self, topic_name: &str) -> String {
        format!("{self}/{topic_name}")
    }
}

impl Hash for MqttTopicPrefix {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            MqttTopicPrefix::Const(v) => v.as_bytes().hash(state),
            MqttTopicPrefix::String(v) => v.as_bytes().hash(state),
        }
    }
}

impl PartialEq for MqttTopicPrefix {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Serialize for MqttTopicPrefix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for MqttTopicPrefix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = MqttTopicPrefix;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "A MQTT topic string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(MqttTopicPrefix::new(v.to_owned()))
            }
        }

        deserializer.deserialize_string(Visitor)
    }
}

impl From<&str> for MqttTopicPrefix {
    fn from(value: &str) -> Self {
        Self::String(value.strip_suffix('/').unwrap_or(value).to_owned())
    }
}

impl std::fmt::Display for MqttTopicPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct MqttService {
    client: AsyncClient,
    subscribers: Arc<Mutex<HashMap<MqttTopicPrefix, broadcast::Sender<Publish>>>>,
}

impl MqttService {
    pub fn new(mqtt_config: &MqttConfig) -> (Self, JoinHandle<()>) {
        let mut options = MqttOptions::new(
            mqtt_config.endpoint_name.as_str(),
            mqtt_config.host.as_str(),
            mqtt_config.port,
        );
        options.set_credentials(mqtt_config.username.clone(), mqtt_config.password.clone());

        let (client, mut event_loop) = AsyncClient::new(options, 10);

        let subscribers = Arc::new(Mutex::new(HashMap::<
            MqttTopicPrefix,
            broadcast::Sender<Publish>,
        >::new()));

        let subscribers_inner = subscribers.clone();
        let task = tokio::task::spawn(async move {
            loop {
                let event = match event_loop.poll().await {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("MQTT event loop error: {e}");
                        continue;
                    }
                };

                if let Event::Incoming(Incoming::Publish(message)) = event {
                    let prefix = MqttTopicPrefix::from_topic_name(&message.topic)
                        .unwrap_or(EMPTY_TOPIC_PREFIX.clone());

                    if let Some(subscriber) = subscribers_inner.clone().lock().get(&prefix) {
                        let _ = subscriber.send(message);
                    }
                }
            }
        });

        (
            Self {
                client,
                subscribers,
            },
            task,
        )
    }

    pub fn receiver(&self, topic_prefix: MqttTopicPrefix) -> broadcast::Receiver<Publish> {
        let mut guard = self.subscribers.lock();

        let subscriber = guard
            .entry(topic_prefix)
            .or_insert_with(|| broadcast::channel(100).0);

        subscriber.subscribe()
    }

    pub async fn subscribe(&self, topic: &str) -> Result<(), Error> {
        let prefix = MqttTopicPrefix::from_topic_name(topic).unwrap_or(EMPTY_TOPIC_PREFIX.clone());
        self.subscribers
            .lock()
            .entry(prefix)
            .or_insert_with(|| broadcast::channel(100).0);

        self.client.subscribe(topic, QoS::AtMostOnce).await?;

        Ok(())
    }

    pub async fn publish(&self, topic_name: &str, payload: &[u8]) -> Result<(), Error> {
        self.client
            .publish(topic_name, QoS::AtMostOnce, false, payload)
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("MQTT client error: {0}")]
    Mqtt(#[from] rumqttc::ClientError),
    #[error("MQTT connection error: {0}")]
    MqttConnection(#[from] rumqttc::ConnectionError),
}
