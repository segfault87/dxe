use futures::stream;
use influxdb2::Client;
use influxdb2::models::DataPoint;
pub use influxdb2::models::FieldValue;
use num_traits::Float;

use crate::config::InfluxDbConfig;

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

    pub async fn publish_values<F: Float>(
        &self,
        measurement: &str,
        tags: impl Iterator<Item = (&str, String)>,
        values: impl Iterator<Item = (&str, F)>,
    ) -> Result<(), Error> {
        let mut datapoint = DataPoint::builder(measurement);

        for (name, value) in tags {
            datapoint = datapoint.tag(name, value);
        }
        for (name, value) in values {
            if let Some(value) = value.to_f64() {
                datapoint = datapoint.field(name, value);
            }
        }

        let datapoint = datapoint.build()?;

        Ok(self
            .client
            .write(&self.bucket, stream::iter([datapoint]))
            .await?)
    }

    pub async fn publish_event<T: Into<FieldValue>>(
        &self,
        measurement: &str,
        tags: impl Iterator<Item = (&str, String)>,
        values: impl Iterator<Item = (&str, T)>,
    ) -> Result<(), Error> {
        let mut datapoint = DataPoint::builder(measurement);

        for (name, value) in tags {
            datapoint = datapoint.tag(name, value);
        }
        for (name, value) in values {
            datapoint = datapoint.field(name, value.into());
        }

        let datapoint = datapoint.build()?;

        Ok(self
            .client
            .write(&self.bucket, stream::iter([datapoint]))
            .await?)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("InfluxDB request error: {0}")]
    RequestError(#[from] influxdb2::RequestError),
    #[error("InfluxDB datapoint error: {0}")]
    DatapointError(#[from] influxdb2::models::data_point::DataPointError),
}
