use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use chrono::{DateTime, Local, TimeDelta, Utc};
use dxe_s2s_shared::Timestamp;
use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_s2s_shared::handlers::{BookingType, GetBookingsResponse};
use dxe_types::{BookingId, UnitId};
use parking_lot::Mutex;
use tokio_task_scheduler::{Scheduler, Task, TaskBuilder};

use crate::client::DxeClient;
use crate::config::events::{BookingEventConfig, BookingEventType};
use crate::events::{Event, EventSender};
use crate::tables::{QualifiedPath, TablePublisher};
use crate::types::{BookingEventId, Endpoint, EventId, PublishKey};

static PUBLISH_KEY_IS_ACTIVE: PublishKey = PublishKey::new_const("is_active");
static PUBLISH_KEY_IS_ACTIVE_INCL_OFFSETS: PublishKey =
    PublishKey::new_const("is_active_incl_offsets");

pub struct BookingsPath;

impl QualifiedPath for BookingsPath {
    type TableKey = UnitId;
    type Path = Endpoint;

    fn path(table_key: &Self::TableKey) -> Self::Path {
        Endpoint::Bookings(table_key.clone())
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct BookingTaskKey(BookingEventId, BookingId);

impl Display for BookingTaskKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.1, self.0)
    }
}

struct BookingTask {
    booking: BookingWithUsers,
    time: DateTime<Utc>,
    r#type: BookingEventType,
    is_active: Option<bool>,
    is_active_offset_edge: Option<bool>,
}

pub struct BookingStateManager {
    config: HashMap<BookingEventId, BookingEventConfig>,
    offsets: HashMap<UnitId, (TimeDelta, TimeDelta)>,

    event_sender: EventSender,
    scheduler: Scheduler,
    client: DxeClient,

    booking_entries: Arc<Mutex<HashMap<BookingId, BookingWithUsers>>>,
    pending_tasks: Mutex<HashMap<BookingTaskKey, BookingTask>>,
    table: TablePublisher<UnitId, Endpoint, BookingsPath>,
}

impl BookingStateManager {
    pub fn new(
        config: &HashMap<BookingEventId, BookingEventConfig>,
        event_sender: EventSender,
        client: DxeClient,
        scheduler: Scheduler,
    ) -> Self {
        let mut offsets = HashMap::new();

        for config in config.values() {
            for unit_id in config.unit_ids.iter() {
                let entry: &mut (TimeDelta, TimeDelta) =
                    offsets.entry(unit_id.clone()).or_default();
                match config.r#type {
                    BookingEventType::OnStart => {
                        if entry.0 > config.offset {
                            entry.0 = config.offset;
                        }
                    }
                    BookingEventType::OnEnd => {
                        if entry.1 < config.offset {
                            entry.1 = config.offset;
                        }
                    }
                }
            }
        }

        Self {
            config: config.clone(),
            offsets,
            event_sender,
            scheduler,
            client,
            booking_entries: Arc::new(Mutex::new(HashMap::new())),
            pending_tasks: Mutex::new(HashMap::new()),
            table: TablePublisher::new(),
        }
    }

    pub fn get_states(&self) -> Arc<Mutex<HashMap<BookingId, BookingWithUsers>>> {
        self.booking_entries.clone()
    }

    async fn update(self: Arc<Self>) {
        let response = match self
            .client
            .get::<GetBookingsResponse>(
                "/pending-bookings",
                Some(&format!("type={}", BookingType::Confirmed)),
            )
            .await
        {
            Ok(v) => v,
            Err(e) => {
                log::error!("Could not get pending bookings: {e}");
                return;
            }
        };

        let now = Utc::now();

        let mut current_entries: HashMap<BookingId, BookingWithUsers> = HashMap::new();
        let mut tasks: HashMap<BookingTaskKey, BookingTask> = HashMap::new();
        let mut prev_entries: HashMap<UnitId, Timestamp> = HashMap::new();
        for (unit_id, mut bookings) in response.bookings.into_iter() {
            let Some((offset_start, offset_end)) = self.offsets.get(&unit_id).cloned() else {
                continue;
            };
            bookings.sort_by_key(|v| v.booking.date_start.clone());

            let mut is_active = false;
            let mut is_active_incl_offsets = false;

            for booking in bookings {
                let start = booking.booking.date_start.to_utc();
                let end = booking.booking.date_end.to_utc();

                if !is_active && now >= start && end > now {
                    is_active = true;
                }
                if now >= start + std::cmp::min(offset_start, TimeDelta::zero())
                    && end + std::cmp::max(offset_end, TimeDelta::zero()) > now
                {
                    if !is_active_incl_offsets {
                        is_active_incl_offsets = true;
                    }

                    current_entries.insert(booking.booking.id, booking.clone());
                }

                let is_continous = if let Some(prev) = prev_entries.get(&unit_id) {
                    prev == &booking.booking.date_start
                } else {
                    false
                };

                for (event_id, config) in self.config.iter() {
                    if !config.unit_ids.contains(&unit_id) {
                        continue;
                    }

                    let (is_active, is_active_offset_edge, time) = match config.r#type {
                        BookingEventType::OnStart => {
                            let is_active = if config.offset.is_zero() {
                                Some(true)
                            } else {
                                None
                            };
                            let is_active_offset_edge = if config.offset == offset_start {
                                Some(true)
                            } else {
                                None
                            };
                            (
                                is_active,
                                is_active_offset_edge,
                                booking.booking.date_start.to_utc() + config.offset,
                            )
                        }
                        BookingEventType::OnEnd => {
                            let is_active = if config.offset.is_zero() {
                                Some(false)
                            } else {
                                None
                            };
                            let is_active_offset_edge = if config.offset == offset_start {
                                Some(false)
                            } else {
                                None
                            };
                            (
                                is_active,
                                is_active_offset_edge,
                                booking.booking.date_end.to_utc() + config.offset,
                            )
                        }
                    };

                    let delta = time - now;
                    if delta.num_seconds() < 0 || delta.num_days() >= 1 {
                        continue;
                    }
                    if let Some(continuation) = config.continuation
                        && continuation != is_continous
                    {
                        continue;
                    }

                    tasks.insert(
                        BookingTaskKey(event_id.clone(), booking.booking.id),
                        BookingTask {
                            booking: booking.clone(),
                            time,
                            r#type: config.r#type,
                            is_active,
                            is_active_offset_edge,
                        },
                    );
                }

                prev_entries.insert(unit_id.clone(), booking.booking.date_end.clone());
            }

            self.table.update_value(
                unit_id.clone(),
                PUBLISH_KEY_IS_ACTIVE.clone(),
                serde_json::Value::Bool(is_active),
            );
            self.table.update_value(
                unit_id.clone(),
                PUBLISH_KEY_IS_ACTIVE_INCL_OFFSETS.clone(),
                serde_json::Value::Bool(is_active_incl_offsets),
            );
        }
        drop(prev_entries);

        *self.booking_entries.lock() = current_entries;

        let tasks_to_remove = {
            let mut pending_tasks = self.pending_tasks.lock();

            pending_tasks
                .extract_if(|k, v| {
                    if matches!(v.r#type, BookingEventType::OnEnd)
                        && v.time - now
                            < self
                                .offsets
                                .get(&v.booking.booking.unit_id)
                                .map(|v| v.1)
                                .unwrap_or_default()
                    {
                        return false;
                    }
                    tasks
                        .get(k)
                        .map(|new_v| new_v.time != v.time)
                        .unwrap_or(true)
                })
                .collect::<HashMap<_, _>>()
        };

        for key in tasks_to_remove.keys() {
            let task_name = key.to_string();
            log::info!("Removing booking task {task_name}...");
            let _ = Arc::clone(&self).scheduler.remove(task_name.as_str()).await;
        }

        for (key, value) in tasks
            .iter()
            .filter(|(k, _)| !self.pending_tasks.lock().contains_key(k))
        {
            let task_name = key.to_string();
            log::info!("Scheduling booking task {task_name} at {}...", value.time);

            let task_name_cloned = task_name.clone();
            let booking_cloned = value.booking.clone();
            let arc_self = Arc::clone(&self);
            let key_cloned = key.0.clone();
            let is_active = value.is_active;
            let is_active_offset_edge = value.is_active_offset_edge;
            let task = TaskBuilder::new(task_name.as_str(), move || {
                if let Some(is_active) = is_active {
                    arc_self.clone().table.update_value(
                        booking_cloned.booking.unit_id.clone(),
                        PUBLISH_KEY_IS_ACTIVE.clone(),
                        serde_json::Value::Bool(is_active),
                    );
                }
                if let Some(is_active_offset_edge) = is_active_offset_edge {
                    arc_self.clone().table.update_value(
                        booking_cloned.booking.unit_id.clone(),
                        PUBLISH_KEY_IS_ACTIVE_INCL_OFFSETS.clone(),
                        serde_json::Value::Bool(is_active_offset_edge),
                    );
                }

                arc_self.clone().event_sender.publish(
                    EventId::Booking(key_cloned.clone()),
                    Event::Booking {
                        booking: booking_cloned.clone(),
                    },
                );

                let scheduler = arc_self.scheduler.clone();
                let task_name = task_name_cloned.clone();
                tokio::task::spawn(async move {
                    let _ = scheduler.remove(task_name.as_str()).await;
                });

                Ok(())
            })
            .daily()
            .at(value
                .time
                .with_timezone(&Local)
                .format("%H:%M:%S")
                .to_string()
                .as_str())
            .unwrap()
            .build();

            if let Err(e) = self.scheduler.add_task(task).await {
                log::error!("Could not schedule {task_name}: {e}");
            }
        }

        *Arc::clone(&self).pending_tasks.lock() = tasks;
    }

    pub fn task(self) -> Task {
        let arc_self = Arc::new(self);

        TaskBuilder::new("booking_state_manager", move || {
            let cloned_arc_self = arc_self.clone();
            tokio::task::spawn(async move {
                cloned_arc_self.update().await;
            });

            Ok(())
        })
        .every_minutes(5)
        .build()
    }

    pub fn publisher(&self) -> TablePublisher<UnitId, Endpoint, BookingsPath> {
        self.table.clone()
    }
}
