use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use parking_lot::Mutex;
use serde::Deserialize;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

use crate::types::{PublishKey, PublishedValues};

const BROADCAST_CHANNEL_SIZE: usize = 10;

pub trait StateTable {
    type Key;

    fn get(&self, key: &Self::Key) -> Option<serde_json::Value>;
}

pub trait QualifiedPath {
    type TableKey;
    type Path;

    fn path(table_key: &Self::TableKey) -> Self::Path;
}

#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct SingleTable(PublishedValues);

impl SingleTable {
    pub fn get(&self, key: &PublishKey) -> Option<serde_json::Value> {
        self.0.get(key).cloned()
    }

    pub fn insert(&mut self, key: PublishKey, value: serde_json::Value) {
        self.0.insert(key, value);
    }

    pub fn update_values(&mut self, values: impl Iterator<Item = (PublishKey, serde_json::Value)>) {
        self.0.extend(values);
    }
}

impl From<PublishedValues> for SingleTable {
    fn from(value: PublishedValues) -> Self {
        Self(value)
    }
}

impl StateTable for SingleTable {
    type Key = PublishKey;

    fn get(&self, key: &Self::Key) -> Option<serde_json::Value> {
        self.0.get(key).cloned()
    }
}

impl StateTable for PublishedValues {
    type Key = PublishKey;

    fn get(&self, key: &Self::Key) -> Option<serde_json::Value> {
        self.get(key).cloned()
    }
}

pub struct Table<K> {
    table: HashMap<K, SingleTable>,
}

impl<K> Default for Table<K> {
    fn default() -> Self {
        Self {
            table: HashMap::new(),
        }
    }
}

impl<K: Eq + Hash> StateTable for Table<K> {
    type Key = (K, PublishKey);

    fn get(&self, (table_key, key): &Self::Key) -> Option<serde_json::Value> {
        self.get_table(table_key).and_then(|table| table.get(key))
    }
}

impl<K: Eq + Hash> Table<K> {
    pub fn get_table(&self, table_key: &K) -> Option<&SingleTable> {
        self.table.get(table_key)
    }

    pub fn replace(&mut self, table_key: K, table: SingleTable) {
        self.table.insert(table_key, table);
    }

    pub fn update(&mut self, table_key: K, values: PublishedValues) {
        self.table
            .entry(table_key)
            .or_default()
            .update_values(values.into_iter());
    }

    pub fn update_value(&mut self, table_key: K, key: PublishKey, value: serde_json::Value) {
        self.table.entry(table_key).or_default().insert(key, value);
    }
}

pub struct TableUpdateReceiver {
    receiver: BroadcastStream<PublishedValues>,
    gc: Option<Box<dyn FnOnce() + Send>>,
}

impl Stream for TableUpdateReceiver {
    type Item = Result<PublishedValues, BroadcastStreamRecvError>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let inner = Pin::new(&mut this.receiver);
        inner.poll_next(cx)
    }
}

// Garbage collection
impl Drop for TableUpdateReceiver {
    fn drop(&mut self) {
        if let Some(gc) = self.gc.take() {
            gc();
        }
    }
}

pub struct TablePublisher<K, P, QP: QualifiedPath<TableKey = K, Path = P>> {
    table: Arc<Mutex<Table<K>>>,
    broadcasts: Arc<Mutex<HashMap<P, broadcast::Sender<PublishedValues>>>>,
    _phantom: PhantomData<QP>,
}

impl<K, P, QP: QualifiedPath<TableKey = K, Path = P>> Clone for TablePublisher<K, P, QP> {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
            broadcasts: self.broadcasts.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<K: Eq + Hash, P: Eq + Hash + Clone + Send + 'static, QP: QualifiedPath<TableKey = K, Path = P>>
    TablePublisher<K, P, QP>
{
    pub fn new() -> Self {
        Self {
            table: Arc::new(Mutex::new(Table::default())),
            broadcasts: Arc::new(Mutex::new(HashMap::new())),
            _phantom: PhantomData,
        }
    }

    pub fn get_values(&self, table_key: &K) -> Option<PublishedValues> {
        self.table.lock().get_table(table_key).map(|v| v.0.clone())
    }

    pub fn replace(&self, table_key: K, table: SingleTable) {
        let guard = self.broadcasts.lock();
        if let Some(broadcast) = guard.get(&QP::path(&table_key)) {
            let _ = broadcast.send(table.0.clone());
        }

        self.table.lock().replace(table_key, table);
    }

    pub fn update(&self, table_key: K, values: PublishedValues) {
        let guard = self.broadcasts.lock();
        if let Some(broadcast) = guard.get(&QP::path(&table_key)) {
            let _ = broadcast.send(values.clone());
        }

        self.table.lock().update(table_key, values);
    }

    pub fn update_value(&self, table_key: K, key: PublishKey, value: serde_json::Value) {
        let guard = self.broadcasts.lock();
        if let Some(broadcast) = guard.get(&QP::path(&table_key)) {
            let _ = broadcast.send(HashMap::from([(key.clone(), value.clone())]));
        }

        self.table.lock().update_value(table_key, key, value);
    }

    pub fn subscribe(&self, table_key: K) -> TableUpdateReceiver {
        let mut guard = self.broadcasts.lock();

        let path = QP::path(&table_key);

        let sender = guard
            .entry(path.clone())
            .or_insert_with(|| broadcast::channel(BROADCAST_CHANNEL_SIZE).0);

        let receiver = sender.subscribe();

        let broadcasts = Arc::downgrade(&self.broadcasts);
        TableUpdateReceiver {
            receiver: BroadcastStream::new(receiver),
            gc: Some(Box::new(move || {
                if let Some(broadcasts) = broadcasts.upgrade() {
                    let mut guard = broadcasts.lock();

                    if let Some(sender) = guard.get(&path)
                        && sender.receiver_count() <= 1
                    {
                        guard.remove(&path);
                    }
                }
            })),
        }
    }
}

/*#[derive(Clone)]
pub struct TableCollection {
    z2m_devices: Arc<Mutex<DeviceTable>>,
    sound_meter_devices: Arc<Mutex<DeviceTable>>,
    metrics: Arc<Mutex<MetricTable>>,
}

impl StateTable for TableCollection {
    type Key = EndpointRef;

    fn get(&self, key: &Self::Key) -> Option<serde_json::Value> {
        match &key.endpoint {
            Endpoint::Device(device) => match device.r#type {
                DeviceType::Z2m => self
                    .z2m_devices
                    .lock()
                    .table
                    .get(&device.id)
                    .and_then(|table| table.get(&key.key)),
                DeviceType::SoundMeter => self
                    .sound_meter_devices
                    .lock()
                    .table
                    .get(&device.id)
                    .and_then(|table| table.get(&key.key)),
            },
            Endpoint::Metric(metric_id) => self
                .metrics
                .lock()
                .table
                .get(metric_id)
                .and_then(|table| table.get(&key.key)),
        }
    }
}
*/
