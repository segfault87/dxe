use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local, Utc};
use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_s2s_shared::handlers::{BookingType, GetBookingsResponse};
use dxe_types::{BookingId, UnitId};
use tokio_task_scheduler::{Scheduler, SchedulerError, Task, TaskBuilder};

use crate::callback::EventStateCallback;
use crate::client::DxeClient;

#[derive(Default)]
pub struct BookingStates {
    pub bookings_1d: HashMap<UnitId, Vec<BookingWithUsers>>,
}

pub struct BookingStateManager {
    states: Arc<Mutex<BookingStates>>,
    scheduler: Scheduler,
    client: DxeClient,
    callbacks: Vec<Box<Arc<dyn EventStateCallback<BookingWithUsers> + Send + Sync + 'static>>>,
    pending_tasks: Mutex<HashMap<String, String>>,
}

enum BookingEventType {
    StartWithBuffer,
    Start,
    End,
    EndWithBuffer,
}

fn create_task_name(booking_id: &BookingId, r#type: BookingEventType) -> String {
    let task_name = match r#type {
        BookingEventType::StartWithBuffer => "start_with_buffer",
        BookingEventType::Start => "start",
        BookingEventType::End => "end",
        BookingEventType::EndWithBuffer => "end_with_buffer",
    };

    format!("booking_{booking_id}_{task_name}")
}

fn temp_pending_active_bookings() -> Vec<BookingWithUsers> {
    serde_json::from_str(include_str!("temp.json")).unwrap()
}

impl BookingStateManager {
    pub fn new(client: DxeClient, scheduler: Scheduler) -> (Arc<Mutex<BookingStates>>, Self) {
        let states = Arc::new(Mutex::new(Default::default()));

        (
            states.clone(),
            Self {
                states,
                scheduler,
                client,
                callbacks: Vec::new(),
                pending_tasks: Mutex::new(HashMap::new()),
            },
        )
    }

    pub fn add_callback<T>(&mut self, callback: Arc<T>)
    where
        T: EventStateCallback<BookingWithUsers> + Send + Sync + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    async fn update(self: Arc<Self>) {
        let mut response = match self
            .client
            .get::<GetBookingsResponse>(
                "/pending-bookings",
                Some(&format!("type={}", BookingType::Pending)),
            )
            .await
        {
            Ok(v) => v,
            Err(e) => {
                log::error!("Could not get pending bookings: {e}");
                return;
            }
        };

        response
            .bookings
            .get_mut(&UnitId::from("default".to_owned()))
            .unwrap()
            .extend(temp_pending_active_bookings());

        let now = Utc::now();

        let mut bookings_to_add = vec![];
        let mut bookings_to_delete = vec![];

        {
            let mut states = self.states.lock().unwrap();

            // Remove outdated events
            for bookings in states.bookings_1d.values_mut() {
                bookings.retain(|v| v.booking.date_end_w_buffer.to_utc() < now);
            }

            for (unit_id, bookings) in response.bookings.into_iter() {
                let current_bookings = states.bookings_1d.entry(unit_id.clone()).or_default();

                let mut previous_keys = current_bookings
                    .iter()
                    .map(|v| v.booking.id.clone())
                    .collect::<HashSet<_>>();
                for booking in bookings {
                    let delta = booking.booking.date_start_w_buffer.to_utc() - now;
                    if delta.num_days() != 0 {
                        continue;
                    }

                    previous_keys.remove(&booking.booking.id);
                    if let Some(previous_booking) = current_bookings
                        .iter_mut()
                        .find(|b| b.booking.id == booking.booking.id)
                    {
                        if &booking != previous_booking {
                            *previous_booking = booking;
                        }
                    } else {
                        bookings_to_add.push(booking.clone());
                        current_bookings.push(booking);
                    }
                }

                for booking in current_bookings
                    .iter()
                    .filter(|v| previous_keys.contains(&v.booking.id))
                {
                    bookings_to_delete.push(booking.clone());
                }
                current_bookings.retain(|v| !previous_keys.contains(&v.booking.id));
            }
        }

        for booking in bookings_to_add {
            Arc::clone(&self).on_new_booking(booking, &now).await;
        }
        for booking in bookings_to_delete {
            let booking_start_with_buffer =
                create_task_name(&booking.booking.id, BookingEventType::StartWithBuffer);
            let booking_start = create_task_name(&booking.booking.id, BookingEventType::Start);
            let booking_end = create_task_name(&booking.booking.id, BookingEventType::End);
            let booking_end_with_buffer =
                create_task_name(&booking.booking.id, BookingEventType::EndWithBuffer);

            let mut tasks_to_remove = vec![];

            {
                let mut pending_tasks = self.pending_tasks.lock().unwrap();

                if let Some(id) = pending_tasks.remove(&booking_start_with_buffer) {
                    tasks_to_remove.push(id);
                }
                if let Some(id) = pending_tasks.remove(&booking_start) {
                    tasks_to_remove.push(id);
                }
                if let Some(id) = pending_tasks.remove(&booking_end) {
                    tasks_to_remove.push(id);
                }
                if let Some(id) = pending_tasks.remove(&booking_end_with_buffer) {
                    tasks_to_remove.push(id);
                }
            }

            for task in tasks_to_remove {
                let _ = self.scheduler.remove(&task).await;
            }

            Arc::clone(&self).on_booking_removed(booking, &now).await;
        }

        // Schedule task under 24 hours time frame (if not scheduled)
        let bookings_1d = self.states.lock().unwrap().bookings_1d.clone();

        for (_, bookings) in bookings_1d {
            for booking in bookings {
                let start_with_buffer_task =
                    create_task_name(&booking.booking.id, BookingEventType::StartWithBuffer);
                let start_with_buffer = booking.booking.date_start_w_buffer.to_utc();

                let start_task = create_task_name(&booking.booking.id, BookingEventType::Start);
                let start = booking.booking.date_start.to_utc();

                let end_task = create_task_name(&booking.booking.id, BookingEventType::End);
                let end = booking.booking.date_end.to_utc();

                let end_with_buffer_task =
                    create_task_name(&booking.booking.id, BookingEventType::EndWithBuffer);
                let end_with_buffer = booking.booking.date_end_w_buffer.to_utc();

                {
                    let arc_self = Arc::clone(&self);
                    let booking = booking.clone();
                    let task_name = start_with_buffer_task.clone();

                    Arc::clone(&self)
                        .schedule_task_if_required(
                            start_with_buffer_task.as_str(),
                            start_with_buffer,
                            now.clone(),
                            move || {
                                let arc_self = arc_self.clone();
                                let booking = booking.clone();
                                let task_name = task_name.clone();
                                tokio::task::spawn(async move {
                                    Arc::clone(&arc_self).on_booking_start(booking, true).await;
                                    let task_id = Arc::clone(&arc_self)
                                        .pending_tasks
                                        .lock()
                                        .unwrap()
                                        .remove(&task_name);
                                    if let Some(task_id) = task_id {
                                        let _ = arc_self.scheduler.remove(&task_id).await;
                                    }
                                });

                                Ok(())
                            },
                        )
                        .await;
                }

                {
                    let arc_self = Arc::clone(&self);
                    let booking = booking.clone();
                    let task_name = start_task.clone();

                    Arc::clone(&self)
                        .schedule_task_if_required(
                            start_task.as_str(),
                            start,
                            now.clone(),
                            move || {
                                let arc_self = arc_self.clone();
                                let booking = booking.clone();
                                let task_name = task_name.clone();
                                tokio::task::spawn(async move {
                                    Arc::clone(&arc_self).on_booking_start(booking, false).await;
                                    let task_id = Arc::clone(&arc_self)
                                        .pending_tasks
                                        .lock()
                                        .unwrap()
                                        .remove(&task_name);
                                    if let Some(task_id) = task_id {
                                        let _ = arc_self.scheduler.remove(&task_id).await;
                                    }
                                });

                                Ok(())
                            },
                        )
                        .await;
                }

                {
                    let arc_self = Arc::clone(&self);
                    let booking = booking.clone();
                    let task_name = end_task.clone();

                    Arc::clone(&self)
                        .schedule_task_if_required(end_task.as_str(), end, now.clone(), move || {
                            let arc_self = arc_self.clone();
                            let booking = booking.clone();
                            let task_name = task_name.clone();
                            tokio::task::spawn(async move {
                                Arc::clone(&arc_self).on_booking_end(booking, false).await;
                                let task_id = Arc::clone(&arc_self)
                                    .pending_tasks
                                    .lock()
                                    .unwrap()
                                    .remove(&task_name);
                                if let Some(task_id) = task_id {
                                    let _ = arc_self.scheduler.remove(&task_id).await;
                                }
                            });

                            Ok(())
                        })
                        .await;
                }

                {
                    let arc_self = Arc::clone(&self);
                    let booking = booking.clone();
                    let task_name = end_with_buffer_task.clone();

                    Arc::clone(&self)
                        .schedule_task_if_required(
                            end_with_buffer_task.as_str(),
                            end_with_buffer,
                            now.clone(),
                            move || {
                                let arc_self = arc_self.clone();
                                let booking = booking.clone();
                                let task_name = task_name.clone();
                                tokio::task::spawn(async move {
                                    Arc::clone(&arc_self).on_booking_end(booking, true).await;
                                    let task_id = Arc::clone(&arc_self)
                                        .pending_tasks
                                        .lock()
                                        .unwrap()
                                        .remove(&task_name);
                                    if let Some(task_id) = task_id {
                                        let _ = arc_self.scheduler.remove(&task_id).await;
                                    }
                                });

                                Ok(())
                            },
                        )
                        .await;
                }
            }
        }
    }

    async fn schedule_task_if_required<F>(
        self: Arc<Self>,
        task_name: &str,
        date: DateTime<Utc>,
        now: DateTime<Utc>,
        f: F,
    ) -> bool
    where
        F: Fn() -> Result<(), SchedulerError> + Send + Sync + 'static,
    {
        let delta = date - now;
        if delta.num_seconds() > 0
            && delta.num_days() == 0
            && !self.pending_tasks.lock().unwrap().contains_key(task_name)
        {
            let task = TaskBuilder::new(task_name, f)
                .daily()
                .at(date.format("%H:%M:%S").to_string().as_str())
                .unwrap()
                .build();

            match self.scheduler.add_task(task).await {
                Ok(id) => {
                    log::info!(
                        "Task {task_name} scheduled at {}",
                        date.with_timezone(&Local).format("%H:%M:%S")
                    );
                    self.pending_tasks
                        .lock()
                        .unwrap()
                        .insert(task_name.to_owned(), id);
                    true
                }
                Err(e) => {
                    log::error!("Could not schedule {task_name}: {e}");
                    false
                }
            }
        } else {
            false
        }
    }

    async fn on_booking_start(self: Arc<Self>, booking: BookingWithUsers, buffered: bool) {
        for callback in self.callbacks.iter() {
            if let Err(e) = callback.on_event_start(&booking, buffered).await {
                log::error!("Error while start booking {}: {e}", booking.booking.id);
            }
        }
    }

    async fn on_booking_end(self: Arc<Self>, booking: BookingWithUsers, buffered: bool) {
        for callback in self.callbacks.iter() {
            if let Err(e) = callback.on_event_end(&booking, buffered).await {
                log::error!("Error while end booking {}: {e}", booking.booking.id);
            }
        }
    }

    async fn on_new_booking(self: Arc<Self>, booking: BookingWithUsers, now: &DateTime<Utc>) {
        let start = booking.booking.date_start_w_buffer.to_utc();
        let end = booking.booking.date_end_w_buffer.to_utc();

        let is_in_progress = &start <= now && now < &end;

        for callback in self.callbacks.iter() {
            if let Err(e) = callback.on_event_created(&booking, is_in_progress).await {
                log::error!("Error while create booking {}: {e}", booking.booking.id);
            }
        }
    }

    async fn on_booking_removed(self: Arc<Self>, booking: BookingWithUsers, now: &DateTime<Utc>) {
        let start = booking.booking.date_start_w_buffer.to_utc();
        let end = booking.booking.date_end_w_buffer.to_utc();

        let is_in_progress = &start <= now && now < &end;

        for callback in self.callbacks.iter() {
            if let Err(e) = callback.on_event_deleted(&booking, is_in_progress).await {
                log::error!("Error while deleting booking {}: {e}", booking.booking.id);
            }
        }

        let _ = self.scheduler.remove(&create_task_name(
            &booking.booking.id,
            BookingEventType::Start,
        ));
        let _ = self.scheduler.remove(&create_task_name(
            &booking.booking.id,
            BookingEventType::StartWithBuffer,
        ));
        let _ = self.scheduler.remove(&create_task_name(
            &booking.booking.id,
            BookingEventType::End,
        ));
        let _ = self.scheduler.remove(&create_task_name(
            &booking.booking.id,
            BookingEventType::EndWithBuffer,
        ));
    }

    pub fn task(self) -> Task {
        let arc_self = Arc::new(self);

        TaskBuilder::new("booking_state_manager", move || {
            let arc_self = arc_self.clone();
            tokio::task::spawn(async move {
                arc_self.update().await;
            });

            Ok(())
        })
        .every_minutes(10)
        .build()
    }
}
