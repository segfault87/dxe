use std::sync::Arc;

use num_traits::Float;
use tokio::sync::mpsc;

use crate::services::influxdb::{FieldValue, InfluxDbClient};

pub trait MetricsDataPoint<F: Float, T: Into<String>> {
    fn measurement() -> &'static str;
    fn tags(&self) -> impl Iterator<Item = (&'static str, T)>;
    fn values(&self) -> impl Iterator<Item = (&'static str, F)>;
}

pub trait EventDataPoint<V: Into<FieldValue>, T: Into<String>> {
    fn measurement() -> &'static str;
    fn tags(&self) -> impl Iterator<Item = (&'static str, T)>;
    fn values(&self) -> impl Iterator<Item = (&'static str, V)>;
}

pub enum ExportedDataPoint {
    Metrics(Vec<(&'static str, f64)>),
    Event(Vec<(&'static str, FieldValue)>),
}

pub struct ExportedData {
    measurement: &'static str,
    tags: Vec<(&'static str, String)>,
    datapoint: ExportedDataPoint,
}

#[derive(Clone)]
pub struct MetricsExporterHandle(mpsc::UnboundedSender<ExportedData>);

impl MetricsExporterHandle {
    pub fn export_metrics<F: Float, T: Into<String>, DP: MetricsDataPoint<F, T>>(
        &self,
        metrics: DP,
    ) {
        let datapoint = metrics
            .values()
            .filter_map(|(k, v)| {
                if let Some(v) = v.to_f64() {
                    Some((k, v))
                } else {
                    log::warn!("Metric value for key {k} is not a number.");
                    None
                }
            })
            .collect::<Vec<_>>();

        let exported_data = ExportedData {
            measurement: DP::measurement(),
            tags: metrics.tags().map(|(k, v)| (k, v.into())).collect(),
            datapoint: ExportedDataPoint::Metrics(datapoint),
        };

        let _ = self.0.send(exported_data);
    }

    pub fn export_event<V: Into<FieldValue>, T: Into<String>, DP: EventDataPoint<V, T>>(
        &self,
        event: &DP,
    ) {
        let datapoint = event
            .values()
            .map(|(k, v)| (k, v.into()))
            .collect::<Vec<_>>();

        let exported_data = ExportedData {
            measurement: DP::measurement(),
            tags: event.tags().map(|(k, v)| (k, v.into())).collect(),
            datapoint: ExportedDataPoint::Event(datapoint),
        };

        let _ = self.0.send(exported_data);
    }
}

pub struct MetricsExporter {
    influxdb_client: InfluxDbClient,
    sender: MetricsExporterHandle,
    consumer: Option<mpsc::UnboundedReceiver<ExportedData>>,
}

impl MetricsExporter {
    pub fn new(influxdb_client: InfluxDbClient) -> Self {
        let (sender, consumer) = mpsc::unbounded_channel();

        let sender = MetricsExporterHandle(sender);

        Self {
            influxdb_client,
            sender,
            consumer: Some(consumer),
        }
    }

    pub fn handle(&self) -> MetricsExporterHandle {
        self.sender.clone()
    }

    async fn consumer_loop(self: Arc<Self>, mut consumer: mpsc::UnboundedReceiver<ExportedData>) {
        while let Some(item) = consumer.recv().await {
            match item.datapoint {
                ExportedDataPoint::Metrics(metrics) => {
                    if let Err(e) = self
                        .clone()
                        .influxdb_client
                        .publish_values(
                            item.measurement,
                            item.tags.into_iter(),
                            metrics.into_iter(),
                        )
                        .await
                    {
                        log::warn!("Could not publish metrics to InfluxDB: {e}");
                    }
                }
                ExportedDataPoint::Event(event) => {
                    if let Err(e) = self
                        .clone()
                        .influxdb_client
                        .publish_event(item.measurement, item.tags.into_iter(), event.into_iter())
                        .await
                    {
                        log::warn!("Could not publish metrics to InfluxDB: {e}");
                    }
                }
            }
        }
    }

    pub fn start(mut self) -> (Arc<Self>, tokio::task::JoinHandle<()>) {
        let Some(consumer) = self.consumer.take() else {
            panic!("Consumer has already started");
        };

        let arc_self = Arc::new(self);

        (
            arc_self.clone(),
            tokio::task::spawn(arc_self.consumer_loop(consumer)),
        )
    }
}
