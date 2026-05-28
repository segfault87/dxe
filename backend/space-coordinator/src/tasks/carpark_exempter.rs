use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::Utc;
use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_s2s_shared::handlers::GetAdhocParkingsResponse;
use dxe_types::BookingId;
use parking_lot::Mutex;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::client::DxeClient;
use crate::config::NotificationPriority;
use crate::services::carpark_exemption::CarparkExemptionService;
use crate::services::notification::NotificationService;
use crate::tasks::osd_controller::OsdController;
use crate::tasks::osd_controller::topics::ParkingStates;
use crate::tasks::osd_controller::types::ParkingState;
use crate::tasks::unit_fetcher::UnitsState;

pub struct CarparkExempter {
    client: DxeClient,
    service: CarparkExemptionService,
    notification_service: NotificationService,
    osd_controller: Arc<OsdController>,

    units_state: UnitsState,
    current_bookings: Arc<Mutex<HashMap<BookingId, BookingWithUsers>>>,
}

impl CarparkExempter {
    pub fn new(
        client: DxeClient,
        service: CarparkExemptionService,
        osd_controller: Arc<OsdController>,
        notification_service: NotificationService,
        units_state: UnitsState,
        current_bookings: Arc<Mutex<HashMap<BookingId, BookingWithUsers>>>,
    ) -> Self {
        Self {
            client,
            service,
            notification_service,
            osd_controller,

            units_state,
            current_bookings,
        }
    }

    async fn update(self: Arc<Self>) {
        let mut license_plate_numbers = HashSet::new();

        match self
            .client
            .get::<GetAdhocParkingsResponse>("/adhoc-parkings", None)
            .await
        {
            Ok(parkings) => {
                for parking in parkings.parkings {
                    license_plate_numbers.insert((
                        None,
                        String::new(),
                        String::new(),
                        parking.license_plate_number,
                        false,
                    ));
                }
            }
            Err(e) => {
                log::warn!("Could not fetch adhoc parking information: {e}");
            }
        }

        {
            let now = Utc::now();

            for booking in self.current_bookings.lock().values() {
                let is_current_booking = now >= booking.booking.date_start.to_utc()
                    && booking.booking.date_end.to_utc() > now;
                for user in booking.users.iter() {
                    if let Some(license_plate_number) = &user.license_plate_number
                        && !license_plate_number.is_empty()
                    {
                        license_plate_numbers.insert((
                            Some(booking.booking.unit_id.clone()),
                            booking.booking.customer_name.clone(),
                            user.name.clone(),
                            license_plate_number.clone(),
                            is_current_booking,
                        ));
                    }
                }
            }
        }

        let mut parking_results = HashMap::new();

        for (unit_id, customer_name, user_name, license_plate_number, is_current_booking) in
            license_plate_numbers
        {
            if let Err(e) = match self.service.exempt(&license_plate_number).await {
                Ok((success, entry_date, fuzzy)) => {
                    if let Some(unit_id) = unit_id
                        && let Some(entry_date) = entry_date
                        && is_current_booking
                    {
                        parking_results
                            .entry(unit_id)
                            .or_insert_with(Vec::new)
                            .push(ParkingState {
                                license_plate_number: license_plate_number.clone(),
                                user_name: user_name.clone(),
                                entry_date,
                                exempted: true,
                                fuzzy: fuzzy.clone(),
                            });
                    }

                    if success {
                        self.notification_service.notify(
                            NotificationPriority::Low,
                            format!(
                                "Car parking exempted sucessfully for user {user_name} ({customer_name}){}",
                                if let Some(fuzzy) = fuzzy {
                                    format!(" (recognized as {fuzzy})")
                                } else {
                                    Default::default()
                                }
                            )
                        ).await
                    } else {
                        continue;
                    }
                }
                Err(e) => {
                    self.notification_service
                        .notify(
                            NotificationPriority::Low,
                            format!("Car parking exemption error: {e}"),
                        )
                        .await
                }
            } {
                log::error!("Could not send notification while processing parking exemption: {e}");
            }
        }

        if parking_results.is_empty() {
            for unit_id in self.units_state.get() {
                let _ = self
                    .osd_controller
                    .publish(&ParkingStates {
                        unit_id,
                        states: vec![],
                    })
                    .await;
            }
        }

        for (unit_id, mut states) in parking_results.into_iter() {
            states.sort_by(|a, b| a.user_name.cmp(&b.user_name));
            let parking_states = ParkingStates {
                unit_id: unit_id.clone(),
                states,
            };

            if let Err(e) = self.osd_controller.publish(&parking_states).await {
                log::warn!("Could not publish parking state to OSD: {e}");
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
