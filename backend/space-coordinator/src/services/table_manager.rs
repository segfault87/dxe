use dxe_types::UnitId;

use crate::tables::{StateTable, Table, TablePublisher, TableUpdateReceiver};
use crate::tasks::booking_state_manager::BookingsPath;
use crate::tasks::metrics_publisher::MetricsPath;
use crate::tasks::presence_monitor::PresencePath;
use crate::tasks::sound_meter_controller::SoundMeterPath;
use crate::tasks::z2m_controller::Z2mPath;
use crate::types::{
    DeviceId, DeviceRef, DeviceType, Endpoint, EndpointKey, MetricId, PresenceRef, PublishedValues,
    TenantId,
};

#[derive(Clone)]
struct DevicePublishers {
    z2m_publisher: TablePublisher<DeviceId, DeviceRef, Z2mPath>,
    sound_meter_publisher: TablePublisher<DeviceId, DeviceRef, SoundMeterPath>,
}

impl DevicePublishers {
    fn subscribe(&self, device_ref: DeviceRef) -> TableUpdateReceiver {
        match device_ref.r#type {
            DeviceType::Z2m => self.z2m_publisher.subscribe(device_ref.id),
            DeviceType::SoundMeter => self.sound_meter_publisher.subscribe(device_ref.id),
        }
    }
}

#[derive(Clone)]
pub struct TableManager {
    device_publishers: DevicePublishers,
    metrics_publisher: TablePublisher<MetricId, Endpoint, MetricsPath>,
    presence_publisher: TablePublisher<PresenceRef, Endpoint, PresencePath>,
    bookings_publisher: TablePublisher<UnitId, Endpoint, BookingsPath>,
}

impl TableManager {
    pub fn new(
        z2m_publisher: TablePublisher<DeviceId, DeviceRef, Z2mPath>,
        sound_meter_publisher: TablePublisher<DeviceId, DeviceRef, SoundMeterPath>,
        metrics_publisher: TablePublisher<MetricId, Endpoint, MetricsPath>,
        presence_publisher: TablePublisher<PresenceRef, Endpoint, PresencePath>,
        bookings_publisher: TablePublisher<UnitId, Endpoint, BookingsPath>,
    ) -> Self {
        Self {
            device_publishers: DevicePublishers {
                z2m_publisher,
                sound_meter_publisher,
            },
            metrics_publisher,
            presence_publisher,
            bookings_publisher,
        }
    }

    pub fn subscribe(&self, endpoint: Endpoint) -> TableUpdateReceiver {
        match endpoint {
            Endpoint::Device(device_ref) => self.device_publishers.subscribe(device_ref),
            Endpoint::Metric(metric_id) => self.metrics_publisher.subscribe(metric_id),
            Endpoint::Presence(presence_ref) => self.presence_publisher.subscribe(presence_ref),
            Endpoint::Bookings(unit_id) => self.bookings_publisher.subscribe(unit_id),
        }
    }
}

#[derive(Default)]
pub struct TableSnapshot {
    z2m_table: Table<DeviceId>,
    sound_meter_table: Table<DeviceId>,
    metric_table: Table<MetricId>,
    presence_global_table: Table<()>,
    presence_table: Table<TenantId>,
    bookings_table: Table<UnitId>,
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
            Endpoint::Presence(PresenceRef::Global) => {
                self.presence_global_table.update((), values)
            }
            Endpoint::Presence(PresenceRef::Tenant(tenant_id)) => {
                self.presence_table.update(tenant_id.clone(), values)
            }
            Endpoint::Bookings(unit_id) => self.bookings_table.update(unit_id.clone(), values),
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
            Endpoint::Presence(PresenceRef::Global) => {
                self.presence_global_table.get(&((), key.key.clone()))
            }
            Endpoint::Presence(PresenceRef::Tenant(tenant_id)) => self
                .presence_table
                .get(&(tenant_id.clone(), key.key.clone())),
            Endpoint::Bookings(unit_id) => {
                self.bookings_table.get(&(unit_id.clone(), key.key.clone()))
            }
        }
    }
}
