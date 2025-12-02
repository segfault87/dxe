use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::{DateTime, Local, Utc};
use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_s2s_shared::handlers::{BookingType, GetBookingsResponse};
use dxe_types::{BookingId, UnitId};
use parking_lot::Mutex;
use tokio_task_scheduler::{Scheduler, SchedulerError, Task, TaskBuilder};

use crate::callback::EventStateCallback;
use crate::client::DxeClient;

#[derive(Default)]
pub struct BookingStates {
    pub bookings_1d: HashMap<UnitId, Vec<BookingWithUsers>>,
    pub has_initialized: bool,
}

pub struct BookingStateManager {
    states: Arc<Mutex<BookingStates>>,

    scheduler: Scheduler,
    client: DxeClient,
    callbacks: Vec<Arc<dyn EventStateCallback<BookingWithUsers> + Send + Sync + 'static>>,
    pending_tasks: Mutex<HashMap<String, String>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BookingPhase {
    StartWithBuffer,
    Start,
    End,
    EndWithBuffer,
}

fn create_task_name(booking_id: &BookingId, phase: BookingPhase) -> String {
    let task_name = match phase {
        BookingPhase::StartWithBuffer => "start_with_buffer",
        BookingPhase::Start => "start",
        BookingPhase::End => "end",
        BookingPhase::EndWithBuffer => "end_with_buffer",
    };

    format!("booking_{booking_id}_{task_name}")
}

fn temp_pending_active_bookings() -> Vec<BookingWithUsers> {
    serde_json::from_str(include_str!("../../booking_overrides.json")).unwrap()
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
        self.callbacks.push(callback);
    }

    async fn update(self: Arc<Self>) {
        let mut response = match self
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

        response
            .bookings
            .get_mut(&UnitId::from("default".to_owned()))
            .unwrap()
            .extend(temp_pending_active_bookings());

        let now = Utc::now();

        let mut bookings_to_add = vec![];
        let mut bookings_to_delete = vec![];

        {
            let mut states = self.states.lock();

            // Remove outdated events
            for bookings in states.bookings_1d.values_mut() {
                bookings.retain(|v| v.booking.date_end_w_buffer.to_utc() > now);
            }

            for (unit_id, bookings) in response.bookings.into_iter() {
                let current_bookings = states.bookings_1d.entry(unit_id.clone()).or_default();

                let mut previous_keys = current_bookings
                    .iter()
                    .map(|v| v.booking.id)
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

            if !states.has_initialized {
                states.has_initialized = true;

                for callback in self.callbacks.iter() {
                    callback.on_initialized();
                }
            }
        }

        for booking in bookings_to_add {
            Arc::clone(&self).on_new_booking(booking, &now).await;
        }
        for booking in bookings_to_delete {
            let booking_start_with_buffer =
                create_task_name(&booking.booking.id, BookingPhase::StartWithBuffer);
            let booking_start = create_task_name(&booking.booking.id, BookingPhase::Start);
            let booking_end = create_task_name(&booking.booking.id, BookingPhase::End);
            let booking_end_with_buffer =
                create_task_name(&booking.booking.id, BookingPhase::EndWithBuffer);

            let mut tasks_to_remove = vec![];

            {
                let mut pending_tasks = self.pending_tasks.lock();

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
        let bookings_1d = self.states.lock().bookings_1d.clone();

        for (_, bookings) in bookings_1d {
            for booking in bookings {
                let start_with_buffer_task =
                    create_task_name(&booking.booking.id, BookingPhase::StartWithBuffer);
                let start_with_buffer = booking.booking.date_start_w_buffer.to_utc();

                let start_task = create_task_name(&booking.booking.id, BookingPhase::Start);
                let start = booking.booking.date_start.to_utc();

                let end_task = create_task_name(&booking.booking.id, BookingPhase::End);
                let end = booking.booking.date_end.to_utc();

                let end_with_buffer_task =
                    create_task_name(&booking.booking.id, BookingPhase::EndWithBuffer);
                let end_with_buffer = booking.booking.date_end_w_buffer.to_utc();

                {
                    let arc_self = Arc::clone(&self);
                    let booking = booking.clone();
                    let task_name = start_with_buffer_task.clone();

                    Arc::clone(&self)
                        .schedule_task_if_required(
                            start_with_buffer_task.as_str(),
                            start_with_buffer,
                            now,
                            move || {
                                log::info!("booking_start_with_buffer event triggered.");
                                let arc_self = arc_self.clone();
                                let booking = booking.clone();
                                let task_name = task_name.clone();
                                tokio::task::spawn(async move {
                                    Arc::clone(&arc_self).on_booking_start(booking, true).await;
                                    let task_id = Arc::clone(&arc_self)
                                        .pending_tasks
                                        .lock()
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
                        .schedule_task_if_required(start_task.as_str(), start, now, move || {
                            log::info!("booking_start event triggered.");
                            let arc_self = arc_self.clone();
                            let booking = booking.clone();
                            let task_name = task_name.clone();
                            tokio::task::spawn(async move {
                                Arc::clone(&arc_self).on_booking_start(booking, false).await;
                                let task_id = Arc::clone(&arc_self)
                                    .pending_tasks
                                    .lock()
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
                    let task_name = end_task.clone();

                    Arc::clone(&self)
                        .schedule_task_if_required(end_task.as_str(), end, now, move || {
                            log::info!("booking_end event triggered.");
                            let arc_self = arc_self.clone();
                            let booking = booking.clone();
                            let task_name = task_name.clone();
                            tokio::task::spawn(async move {
                                Arc::clone(&arc_self).on_booking_end(booking, false).await;
                                let task_id = Arc::clone(&arc_self)
                                    .pending_tasks
                                    .lock()
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
                            now,
                            move || {
                                log::info!("booking_end_with_buffer event triggered.");
                                let arc_self = arc_self.clone();
                                let booking = booking.clone();
                                let task_name = task_name.clone();
                                tokio::task::spawn(async move {
                                    Arc::clone(&arc_self).on_booking_end(booking, true).await;
                                    let task_id = Arc::clone(&arc_self)
                                        .pending_tasks
                                        .lock()
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
            && !self.pending_tasks.lock().contains_key(task_name)
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
                    self.pending_tasks.lock().insert(task_name.to_owned(), id);
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
        let start_w_buffer = booking.booking.date_start_w_buffer.to_utc();
        let start = booking.booking.date_start.to_utc();
        let end = booking.booking.date_end.to_utc();
        let end_w_buffer = booking.booking.date_end_w_buffer.to_utc();

        let is_in_progress = &start_w_buffer <= now && now < &end_w_buffer;

        if is_in_progress {
            let has_started = &start < now && now < &end;

            for callback in self.callbacks.iter() {
                if let Err(e) = callback.on_event_start(&booking, true).await {
                    log::error!("Error while starting booking {}: {e}", booking.booking.id);
                }
                if has_started && let Err(e) = callback.on_event_start(&booking, false).await {
                    log::error!("Error while starting booking {}: {e}", booking.booking.id);
                }
            }
        }
    }

    async fn on_booking_removed(self: Arc<Self>, booking: BookingWithUsers, now: &DateTime<Utc>) {
        let start_w_buffer = booking.booking.date_start_w_buffer.to_utc();
        let end = booking.booking.date_end.to_utc();
        let end_w_buffer = booking.booking.date_end_w_buffer.to_utc();

        let is_in_progress = &start_w_buffer <= now && now < &end_w_buffer;

        if is_in_progress {
            let is_in_buffer = now >= &end;

            for callback in self.callbacks.iter() {
                if !is_in_buffer && let Err(e) = callback.on_event_end(&booking, false).await {
                    log::error!("Error while stopping booking {}: {e}", booking.booking.id);
                }
                if let Err(e) = callback.on_event_end(&booking, true).await {
                    log::error!("Error while stopping booking {}: {e}", booking.booking.id);
                }
            }
        }

        let _ = self
            .scheduler
            .remove(&create_task_name(&booking.booking.id, BookingPhase::Start))
            .await;
        let _ = self
            .scheduler
            .remove(&create_task_name(
                &booking.booking.id,
                BookingPhase::StartWithBuffer,
            ))
            .await;
        let _ = self
            .scheduler
            .remove(&create_task_name(&booking.booking.id, BookingPhase::End))
            .await;
        let _ = self
            .scheduler
            .remove(&create_task_name(
                &booking.booking.id,
                BookingPhase::EndWithBuffer,
            ))
            .await;
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
