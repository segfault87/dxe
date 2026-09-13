use futures::{Stream, StreamExt};
use influxdb2::Client;
use influxdb2::models::{DataPoint, FieldValue};

use crate::config::InfluxDbConfig;

pub enum MetricValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl From<MetricValue> for FieldValue {
    fn from(value: MetricValue) -> Self {
        match value {
            MetricValue::Boolean(v) => FieldValue::Bool(v),
            MetricValue::Float(v) => FieldValue::F64(v),
            MetricValue::Integer(v) => FieldValue::I64(v),
            MetricValue::String(v) => FieldValue::String(v),
        }
    }
}

impl From<bool> for MetricValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i64> for MetricValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for MetricValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<String> for MetricValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

pub struct Publishment {
    pub measurement: &'static str,
    pub tags: Vec<(&'static str, String)>,
    pub fields: Vec<(&'static str, MetricValue)>,
}

pub struct InfluxDbClient {
    client: Client,

    bucket: String,
}

impl InfluxDbClient {
    pub fn new(config: &InfluxDbConfig) -> Self {
        Self {
            client: Client::new(config.url.clone(), &config.org, &config.token),
            bucket: config.bucket.clone(),
        }
    }

    pub async fn publish_loop<S: Stream<Item = Publishment> + Send + Sync + Unpin + 'static>(
        &self,
        mut stream: S,
    ) -> Result<(), Error> {
        while let Some(item) = stream.next().await {
            let mut datapoint = DataPoint::builder(item.measurement);

            for (name, value) in item.tags {
                datapoint = datapoint.tag(name, value);
            }
            for (name, value) in item.fields {
                datapoint = datapoint.field(name, value);
            }

            let datapoint = datapoint.build()?;

            if let Err(e) = self
                .client
                .write(
                    &self.bucket,
                    futures::stream::once(async move { datapoint }),
                )
                .await
            {
                log::warn!("Could not write to InfluxDB: {e}");
            }
        }

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("InfluxDB request error: {0}")]
    RequestError(#[from] influxdb2::RequestError),
    #[error("InfluxDB datapoint error: {0}")]
    DatapointError(#[from] influxdb2::models::data_point::DataPointError),
}
