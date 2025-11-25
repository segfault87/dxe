use std::fmt::Display;

use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::config::MqttConfig;
use crate::config::z2m::Device;

pub type IncomingMessage = (DeviceName, serde_json::Value);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct DeviceName(String);

impl Display for DeviceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct MqttService {
    client: AsyncClient,
    tx: broadcast::Sender<IncomingMessage>,
}

#[derive(Debug)]
pub enum Z2mPublishTopic {
    Get,
    Set,
}

fn get_topic_name(device_name: &DeviceName, command: Option<Z2mPublishTopic>) -> String {
    format!(
        "zigbee2mqtt/{device_name}{}",
        match command {
            None => "",
            Some(Z2mPublishTopic::Get) => "/get",
            Some(Z2mPublishTopic::Set) => "/set",
        }
    )
}

fn get_device_name_from_topic_name(topic: &str) -> Option<DeviceName> {
    topic
        .strip_prefix("zigbee2mqtt/")
        .map(|v| DeviceName(v.to_owned()))
}

impl MqttService {
    pub fn new(mqtt_config: &MqttConfig) -> (Self, JoinHandle<()>) {
        let mut options =
            MqttOptions::new("z2m_client", mqtt_config.host.as_str(), mqtt_config.port);
        options.set_credentials(mqtt_config.username.clone(), mqtt_config.password.clone());

        let (client, mut event_loop) = AsyncClient::new(options, 10);

        let (tx, _rx) = broadcast::channel(100);

        let tx_inner = tx.clone();
        let task = tokio::task::spawn(async move {
            loop {
                let event = match event_loop.poll().await {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("MQTT event loop error: {e}");
                        break;
                    }
                };

                if let Event::Incoming(Incoming::Publish(message)) = event {
                    if let Some(device_name) = get_device_name_from_topic_name(&message.topic) {
                        let Ok(payload) = String::from_utf8(message.payload.to_vec()) else {
                            log::warn!("Failed to parse payload: Invalid UTF-8 string.");
                            continue;
                        };
                        let Ok(payload) = serde_json::from_str::<serde_json::Value>(&payload)
                        else {
                            log::warn!("Failed to parse payload: Invalid JSON string.");
                            continue;
                        };

                        let _ = tx_inner.send((device_name, payload));
                    }
                }
            }
        });

        (Self { client, tx }, task)
    }

    pub fn receiver(&self) -> broadcast::Receiver<IncomingMessage> {
        self.tx.subscribe()
    }

    pub async fn subscribe(&self, device: &Device) -> Result<(), Error> {
        self.client
            .subscribe(get_topic_name(&device.name, None), QoS::AtMostOnce)
            .await?;

        let state_key = device.state_key.clone();

        self.publish(
            &device.name,
            Z2mPublishTopic::Get,
            serde_json::json!({state_key: {}}),
        )
        .await?;

        Ok(())
    }

    pub async fn publish(
        &self,
        device_name: &DeviceName,
        publish_topic: Z2mPublishTopic,
        payload: serde_json::Value,
    ) -> Result<(), Error> {
        self.client
            .publish(
                get_topic_name(device_name, Some(publish_topic)),
                QoS::AtMostOnce,
                false,
                payload.to_string().as_bytes(),
            )
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
