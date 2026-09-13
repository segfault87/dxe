use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::services::influxdb::{InfluxDbClient, MetricValue, Publishment};

pub trait IntoDataPoint {
    fn measurement() -> &'static str;
    fn tags(&self) -> impl Iterator<Item = (&'static str, String)>;
    fn fields(&self) -> impl Iterator<Item = (&'static str, MetricValue)>;
}

pub trait IntoDataPointExt: IntoDataPoint {
    fn to_datapoint(&self) -> Publishment;
}

impl<T: IntoDataPoint> IntoDataPointExt for T {
    fn to_datapoint(&self) -> Publishment {
        Publishment {
            measurement: T::measurement(),
            tags: self.tags().collect(),
            fields: self.fields().collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MetricsExporterHandle(mpsc::UnboundedSender<Publishment>);

impl MetricsExporterHandle {
    pub fn export<DP: IntoDataPoint>(&self, data: &DP) {
        let _ = self.0.send(data.to_datapoint());
    }
}

pub struct MetricsExporter {
    influxdb_client: InfluxDbClient,
    sender: MetricsExporterHandle,
    consumer: Option<mpsc::UnboundedReceiver<Publishment>>,
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

    pub fn start(mut self) -> (Arc<Self>, tokio::task::JoinHandle<()>) {
        let Some(consumer) = self.consumer.take() else {
            panic!("Consumer has already started");
        };

        let arc_self = Arc::new(self);
        let arc_self_cloned = arc_self.clone();

        let consumer = UnboundedReceiverStream::new(consumer);

        let task = tokio::task::spawn(async move {
            if let Err(e) = arc_self
                .clone()
                .influxdb_client
                .publish_loop(consumer)
                .await
            {
                log::error!("Could not publish to InfluxDB: {e}");
            }
        });

        (arc_self_cloned, task)
    }
}
