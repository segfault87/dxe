use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Publish, QoS};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::config::MqttConfig;

#[derive(Debug, Clone)]
pub struct MqttService {
    client: AsyncClient,
    tx: broadcast::Sender<Publish>,
}

impl MqttService {
    pub fn new(mqtt_config: &MqttConfig) -> (Self, JoinHandle<()>) {
        let mut options =
            MqttOptions::new("mqtt_client", mqtt_config.host.as_str(), mqtt_config.port);
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
                        continue;
                    }
                };

                if let Event::Incoming(Incoming::Publish(message)) = event {
                    let _ = tx_inner.send(message);
                }
            }
        });

        (Self { client, tx }, task)
    }

    pub fn receiver(&self) -> broadcast::Receiver<Publish> {
        self.tx.subscribe()
    }

    pub async fn subscribe(&self, topic: &str) -> Result<(), Error> {
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
