use std::collections::HashSet;
use std::error::Error;
use std::sync::{Arc, Mutex};

use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_types::{BookingId, UnitId};
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::callback::EventStateCallback;
use crate::tasks::presence_monitor::PresenceState;

pub struct MqttController {
    unit_id: UnitId,
    presence_state: Arc<Mutex<PresenceState>>,

    active_bookings: Mutex<HashSet<BookingId>>,
}

impl MqttController {
    pub fn new(unit_id: UnitId, presence_state: Arc<Mutex<PresenceState>>) -> Self {
        Self {
            unit_id,
            presence_state,

            active_bookings: Mutex::new(Default::default()),
        }
    }

    async fn update(self: Arc<Self>) {}

    pub fn task(self) -> (Arc<Self>, Task) {
        let task_name = format!("mqtt_controller_{}", self.unit_id);

        let arc_self = Arc::new(self);

        (
            arc_self.clone(),
            TaskBuilder::new(&task_name, move || {
                let arc_self = arc_self.clone();
                tokio::task::spawn(async move {
                    arc_self.update().await;
                });

                Ok(())
            })
            .every_minutes(1)
            .build(),
        )
    }
}

#[async_trait::async_trait]
impl EventStateCallback<BookingWithUsers> for MqttController {
    async fn on_event_created(
        &self,
        event: &BookingWithUsers,
        is_in_progress: bool,
    ) -> Result<(), Box<dyn Error>> {
        if event.booking.unit_id != self.unit_id {
            return Ok(());
        }

        if is_in_progress {
            self.active_bookings
                .lock()
                .unwrap()
                .insert(event.booking.id.clone());
        }

        Ok(())
    }

    async fn on_event_deleted(
        &self,
        event: &BookingWithUsers,
        is_in_progress: bool,
    ) -> Result<(), Box<dyn Error>> {
        if event.booking.unit_id != self.unit_id {
            return Ok(());
        }

        if is_in_progress {
            self.active_bookings
                .lock()
                .unwrap()
                .remove(&event.booking.id);
        }

        Ok(())
    }

    async fn on_event_start(
        &self,
        event: &BookingWithUsers,
        _buffered: bool,
    ) -> Result<(), Box<dyn Error>> {
        if event.booking.unit_id != self.unit_id {
            return Ok(());
        }

        self.active_bookings
            .lock()
            .unwrap()
            .insert(event.booking.id.clone());

        Ok(())
    }

    async fn on_event_end(
        &self,
        event: &BookingWithUsers,
        _buffered: bool,
    ) -> Result<(), Box<dyn Error>> {
        if event.booking.unit_id != self.unit_id {
            return Ok(());
        }

        self.active_bookings
            .lock()
            .unwrap()
            .remove(&event.booking.id);

        Ok(())
    }
}
