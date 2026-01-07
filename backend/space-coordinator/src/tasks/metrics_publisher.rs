use std::collections::HashMap;

use chrono::TimeDelta;
use futures::stream::select_all;
use parking_lot::Mutex;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

use crate::config::metrics::Metric;
use crate::services::table_manager::TableManager;
use crate::tables::{QualifiedPath, TablePublisher};
use crate::types::{Endpoint, MetricId};
use crate::utils::moving_average::MovingAverage;

pub struct MetricsPath;

impl QualifiedPath for MetricsPath {
    type TableKey = MetricId;
    type Path = Endpoint;

    fn path(table_key: &Self::TableKey) -> Self::Path {
        Endpoint::Metric(table_key.clone())
    }
}

pub struct MetricsPublisher {
    metrics: HashMap<MetricId, Metric>,
    table: TablePublisher<MetricId, Endpoint, MetricsPath>,
    publishers: Mutex<HashMap<MetricId, JoinHandle<()>>>,
    started: bool,
}

impl MetricsPublisher {
    pub fn new<'a>(configs: impl Iterator<Item = &'a Metric>) -> Self {
        let table = TablePublisher::new();

        let metrics = configs.map(|v| (v.id.clone(), v.clone())).collect();

        Self {
            metrics,
            table,
            publishers: Mutex::new(Default::default()),
            started: false,
        }
    }

    pub fn publisher(&self) -> TablePublisher<MetricId, Endpoint, MetricsPath> {
        self.table.clone()
    }

    async fn metric_collector_loop(
        metric: Metric,
        table_manager: TableManager,
        publisher: TablePublisher<MetricId, Endpoint, MetricsPath>,
    ) {
        let mut table = metric
            .publish_keys
            .iter()
            .map(|v| {
                (
                    v.clone(),
                    (
                        MovingAverage::<f64>::new(
                            metric.average_window.unwrap_or(TimeDelta::zero()),
                        ),
                        metric
                            .devices
                            .iter()
                            .map(|device| (device.clone(), 0.0f64))
                            .collect::<HashMap<_, _>>(),
                    ),
                )
            })
            .collect::<HashMap<_, _>>();

        let streams = metric
            .devices
            .iter()
            .map(|v| {
                table_manager
                    .subscribe(Endpoint::Device(v.clone()))
                    .map(|item| item.map(|result| (v.clone(), result)))
            })
            .collect::<Vec<_>>();

        let mut combined = select_all(streams);

        while let Some(item) = combined.next().await {
            if let Ok((device_ref, values)) = item {
                let mut values_to_publish = HashMap::new();

                for (publish_key, table) in table.iter_mut() {
                    if let Some(table_value) = values.get(publish_key)
                        && let Some(value) = table_value.as_f64()
                        && let Some(current_value) = table.1.get_mut(&device_ref)
                    {
                        *current_value = table.0.push(value);
                    }

                    values_to_publish.insert(
                        publish_key.clone(),
                        serde_json::json!(table.1.values().copied().sum::<f64>()),
                    );
                }

                publisher.update(metric.id.clone(), values_to_publish);
            }
        }
    }

    pub fn start(&mut self, table_manager: TableManager) {
        if self.started {
            return;
        }

        let mut publishers = self.publishers.lock();
        for metric in self.metrics.values() {
            let handle = tokio::task::spawn(Self::metric_collector_loop(
                metric.clone(),
                table_manager.clone(),
                self.table.clone(),
            ));
            publishers.insert(metric.id.clone(), handle);
        }

        self.started = true;
    }

    pub fn stop(&mut self) {
        if !self.started {
            return;
        }

        let mut guard = self.publishers.lock();
        for task in guard.values_mut() {
            task.abort();
        }
        guard.clear();

        self.started = false;
    }
}

impl Drop for MetricsPublisher {
    fn drop(&mut self) {
        self.stop();
    }
}
