use crate::tables::{StateTable, Table, TablePublisher, TableUpdateReceiver};
use crate::tasks::metrics_publisher::MetricsPath;
use crate::tasks::sound_meter_controller::SoundMeterPath;
use crate::tasks::z2m_controller::Z2mPath;
use crate::types::{
    DeviceId, DeviceRef, DeviceType, Endpoint, EndpointKey, MetricId, PublishedValues,
};

#[derive(Clone)]
struct DevicePublishers {
    z2m_publishers: TablePublisher<DeviceId, DeviceRef, Z2mPath>,
    sound_meter_publishers: TablePublisher<DeviceId, DeviceRef, SoundMeterPath>,
}

impl DevicePublishers {
    fn subscribe(&self, device_ref: DeviceRef) -> TableUpdateReceiver {
        match device_ref.r#type {
            DeviceType::Z2m => self.z2m_publishers.subscribe(device_ref.id),
            DeviceType::SoundMeter => self.sound_meter_publishers.subscribe(device_ref.id),
        }
    }
}

#[derive(Clone)]
pub struct TableManager {
    device_publishers: DevicePublishers,
    metrics_publishers: TablePublisher<MetricId, Endpoint, MetricsPath>,
}

impl TableManager {
    pub fn new(
        z2m_publishers: TablePublisher<DeviceId, DeviceRef, Z2mPath>,
        sound_meter_publishers: TablePublisher<DeviceId, DeviceRef, SoundMeterPath>,
        metrics_publishers: TablePublisher<MetricId, Endpoint, MetricsPath>,
    ) -> Self {
        Self {
            device_publishers: DevicePublishers {
                z2m_publishers,
                sound_meter_publishers,
            },
            metrics_publishers,
        }
    }

    pub fn subscribe(&self, endpoint: Endpoint) -> TableUpdateReceiver {
        match endpoint {
            Endpoint::Device(device_ref) => self.device_publishers.subscribe(device_ref),
            Endpoint::Metric(metric_id) => self.metrics_publishers.subscribe(metric_id),
        }
    }
}

#[derive(Default)]
pub struct TableSnapshot {
    z2m_table: Table<DeviceId>,
    sound_meter_table: Table<DeviceId>,
    metric_table: Table<MetricId>,
}

impl TableSnapshot {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update(&mut self, endpoint: &Endpoint, values: PublishedValues) {
        match endpoint {
            Endpoint::Device(device_ref) => match device_ref.r#type {
                DeviceType::Z2m => self.z2m_table.update(device_ref.id.clone(), values),
                DeviceType::SoundMeter => {
                    self.sound_meter_table.update(device_ref.id.clone(), values)
                }
            },
            Endpoint::Metric(metric_id) => self.metric_table.update(metric_id.clone(), values),
        }
    }
}

impl StateTable for TableSnapshot {
    type Key = EndpointKey;

    fn get(&self, key: &Self::Key) -> Option<serde_json::Value> {
        match &key.endpoint {
            Endpoint::Device(device_ref) => match device_ref.r#type {
                DeviceType::Z2m => self
                    .z2m_table
                    .get(&(device_ref.id.clone(), key.key.clone())),
                DeviceType::SoundMeter => self
                    .sound_meter_table
                    .get(&(device_ref.id.clone(), key.key.clone())),
            },
            Endpoint::Metric(metric_id) => {
                self.metric_table.get(&(metric_id.clone(), key.key.clone()))
            }
        }
    }
}
