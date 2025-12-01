use std::collections::HashSet;
use std::error::Error;
use std::sync::{Arc, Mutex};

use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_types::BookingId;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::callback::EventStateCallback;
use crate::config::AlertPriority;
use crate::services::carpark_exemption::CarparkExemptionService;
use crate::services::notification::NotificationService;
use crate::tasks::booking_state_manager::BookingStates;

pub struct CarparkExempter {
    booking_states: Arc<Mutex<BookingStates>>,
    service: CarparkExemptionService,
    notification_service: NotificationService,

    active_bookings: Mutex<HashSet<BookingId>>,
}

impl CarparkExempter {
    pub fn new(
        booking_states: Arc<Mutex<BookingStates>>,
        service: CarparkExemptionService,
        notification_service: NotificationService,
    ) -> Self {
        Self {
            booking_states,
            service,
            notification_service,

            active_bookings: Mutex::new(HashSet::new()),
        }
    }

    async fn update(self: Arc<Self>) {
        let mut license_plate_numbers = HashSet::new();

        {
            let active_bookings = self.active_bookings.lock().unwrap();

            for bookings in self.booking_states.lock().unwrap().bookings_1d.values() {
                for booking in bookings {
                    if active_bookings.contains(&booking.booking.id) {
                        for user in booking.users.iter() {
                            if let Some(license_plate_number) = &user.license_plate_number
                                && !license_plate_number.is_empty()
                            {
                                license_plate_numbers.insert((
                                    booking.booking.customer_name.clone(),
                                    user.name.clone(),
                                    license_plate_number.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        for (customer_name, user_name, license_plate_number) in license_plate_numbers {
            if let Err(e) = match self.service.exempt(&license_plate_number).await {
                Ok(true) => self.notification_service.notify(AlertPriority::Low, format!("Car parking exempted sucessfully for user {user_name} ({customer_name})")).await,
                Ok(false) => continue,
                Err(e) => self.notification_service.notify(AlertPriority::Low, format!("Car parking exemption error: {e}")).await,
            } {
                log::error!("Could not send notification while processing parking exemption: {e}");
            }
        }
    }

    pub fn task(self) -> (Arc<Self>, Task) {
        let arc_self = Arc::new(self);

        (
            arc_self.clone(),
            TaskBuilder::new("carpark_exempter", move || {
                let arc_self = arc_self.clone();
                tokio::task::spawn(async move {
                    arc_self.update().await;
                });

                Ok(())
            })
            .every_minutes(10)
            .build(),
        )
    }
}

#[async_trait::async_trait]
impl EventStateCallback<BookingWithUsers> for CarparkExempter {
    async fn on_event_start(
        &self,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn Error>> {
        if buffered {
            self.active_bookings
                .lock()
                .unwrap()
                .insert(event.booking.id);
        }

        Ok(())
    }

    async fn on_event_end(
        &self,
        event: &BookingWithUsers,
        buffered: bool,
    ) -> Result<(), Box<dyn Error>> {
        if buffered {
            self.active_bookings
                .lock()
                .unwrap()
                .remove(&event.booking.id);
        }

        Ok(())
    }
}
