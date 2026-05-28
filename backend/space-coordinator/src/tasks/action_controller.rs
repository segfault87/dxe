use std::collections::HashSet;
use std::error::Error as StdError;
use std::sync::Arc;

use dxe_s2s_shared::entities::BookingWithUsers;
use futures::StreamExt;
use futures::stream::select_all;
use parking_lot::Mutex;
use tokio::task::JoinHandle;

use crate::callback::LifecycleEventCallback;
use crate::config::triggers::{StateAction, Trigger, TriggerAction};
use crate::device::SwitchState;
use crate::events::{Event, EventSender};
use crate::services::table_manager::{TableManager, TableSnapshot};
use crate::tasks::booking_reminder::BookingReminder;
use crate::tasks::z2m_controller::Z2mController;
use crate::types::{DeviceRef, Z2mDeviceId};

pub struct DeviceController {
    booking_reminder: Arc<BookingReminder>,
    z2m_controller: Arc<Z2mController>,

    booking_state_callbacks:
        Vec<Arc<dyn LifecycleEventCallback<BookingWithUsers> + Send + Sync + 'static>>,
    osd_state_callbacks:
        Vec<Arc<dyn LifecycleEventCallback<BookingWithUsers> + Send + Sync + 'static>>,
}

impl DeviceController {
    pub fn new(booking_reminder: Arc<BookingReminder>, z2m_controller: Arc<Z2mController>) -> Self {
        Self {
            booking_reminder,
            z2m_controller,

            booking_state_callbacks: Vec::new(),
            osd_state_callbacks: Vec::new(),
        }
    }

    pub fn add_booking_state_callback<T>(&mut self, callback: Arc<T>)
    where
        T: LifecycleEventCallback<BookingWithUsers> + Send + Sync + 'static,
    {
        self.booking_state_callbacks.push(callback);
    }

    pub fn add_osd_state_callback<T>(&mut self, callback: Arc<T>)
    where
        T: LifecycleEventCallback<BookingWithUsers> + Send + Sync + 'static,
    {
        self.osd_state_callbacks.push(callback);
    }

    pub fn build(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub async fn control_booking(
        self: Arc<Self>,
        booking: &BookingWithUsers,
        action: StateAction,
    ) -> Result<(), Box<dyn StdError>> {
        match action {
            StateAction::Start => {
                for callback in self.booking_state_callbacks.iter() {
                    if let Err(e) = callback.clone().on_start(booking).await {
                        log::error!("Error invoking booking start callback: {e}");
                    }
                }
            }
            StateAction::Stop => {
                for callback in self.booking_state_callbacks.iter() {
                    if let Err(e) = callback.clone().on_end(booking).await {
                        log::error!("Error invoking booking end callback: {e}");
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn control_osd(
        self: Arc<Self>,
        booking: &BookingWithUsers,
        action: StateAction,
    ) -> Result<(), Box<dyn StdError>> {
        match action {
            StateAction::Start => {
                for callback in self.osd_state_callbacks.iter() {
                    if let Err(e) = callback.clone().on_start(booking).await {
                        log::error!("Error invoking osd start callback: {e}");
                    }
                }
            }
            StateAction::Stop => {
                for callback in self.osd_state_callbacks.iter() {
                    if let Err(e) = callback.clone().on_end(booking).await {
                        log::error!("Error invoking osd end callback: {e}");
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn send_booking_reminder(
        self: Arc<Self>,
        booking: &BookingWithUsers,
    ) -> Result<(), Box<dyn StdError>> {
        self.booking_reminder.send_reminder(booking).await?;

        Ok(())
    }

    pub async fn control_devices<'a>(
        self: Arc<Self>,
        switches: impl Iterator<Item = (&'a DeviceRef, &'a SwitchState)>,
    ) -> Result<(), Box<dyn StdError>> {
        for (z2m_device_id, state) in switches
            .filter_map(|(k, v)| Some((TryInto::<Z2mDeviceId>::try_into(k.clone()).ok()?, *v)))
        {
            if let Err(e) = self
                .z2m_controller
                .set_switch(z2m_device_id.clone(), state)
                .await
            {
                log::warn!("Could not set switch state for {z2m_device_id}: {e}");
            }
        }

        Ok(())
    }
}

pub struct ActionController {
    triggers: Vec<Trigger>,

    table_subscriber: Mutex<Option<JoinHandle<()>>>,
    table_snapshot: Arc<Mutex<TableSnapshot>>,
    handlers: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl Drop for ActionController {
    fn drop(&mut self) {
        for handle in self.handlers.lock().drain(0..) {
            handle.abort();
        }
        if let Some(handle) = self.table_subscriber.lock().take() {
            handle.abort();
        }
    }
}

impl ActionController {
    pub fn new(triggers: Vec<Trigger>) -> Self {
        Self {
            triggers,

            table_subscriber: Mutex::new(None),
            table_snapshot: Arc::new(Mutex::new(TableSnapshot::new())),
            handlers: Arc::new(Mutex::new(Default::default())),
        }
    }

    fn start_table_receiver(&mut self, table_manager: TableManager) {
        if self.table_subscriber.lock().is_some() {
            log::warn!("Table receiver has already started.");

            return;
        }

        let mut endpoints = HashSet::new();
        for trigger in self.triggers.iter() {
            if let Some(condition) = &trigger.condition {
                endpoints.extend(condition.keys().into_iter().map(|v| v.endpoint));
            }
        }

        let receivers = endpoints
            .into_iter()
            .map(|endpoint| {
                table_manager
                    .subscribe(endpoint.clone())
                    .map(move |item| item.map(|v| (endpoint.clone(), v)))
            })
            .collect::<Vec<_>>();

        let mut combined_receivers = select_all(receivers);

        let snapshot = self.table_snapshot.clone();
        *self.table_subscriber.lock() = Some(tokio::task::spawn(async move {
            while let Some(item) = combined_receivers.next().await {
                if let Ok((endpoint, table)) = item {
                    snapshot.lock().update(&endpoint, table);
                }
            }
        }));
    }

    pub async fn start(
        mut self,
        device_controller: Arc<DeviceController>,
        event_sender: EventSender,
        table_manager: TableManager,
    ) -> Arc<Self> {
        self.start_table_receiver(table_manager);

        let arc_self = Arc::new(self);

        for trigger in arc_self.triggers.iter().cloned() {
            let cloned_event_sender = event_sender.clone();
            let cloned_arc_self = arc_self.clone();
            let cloned_device_controller = device_controller.clone();
            let task = tokio::task::spawn(async move {
                let Trigger {
                    event_ids,
                    condition,
                    actions,
                } = trigger;
                let mut receiver = cloned_event_sender.subscribe(event_ids.into_iter());
                while let Some(item) = receiver.next().await {
                    let (event_id, event) = match item {
                        Ok(v) => v,
                        Err(e) => {
                            log::warn!("Could not fetch event: {e}");
                            break;
                        }
                    };

                    if let Some(condition) = &condition
                        && !matches!(
                            condition.test(&*cloned_arc_self.clone().table_snapshot.lock()),
                            Ok(true)
                        )
                    {
                        continue;
                    }

                    log::info!("Triggering event {event_id}...");

                    for action in actions.iter() {
                        let result = match action {
                            TriggerAction::Switches(switches) => {
                                cloned_device_controller
                                    .clone()
                                    .control_devices(switches.iter())
                                    .await
                            }
                            TriggerAction::Delay(delay) => {
                                tokio::time::sleep(*delay).await;
                                Ok(())
                            }
                            TriggerAction::BookingControl(action) => {
                                let Event::Booking { booking } = &event else {
                                    log::warn!(
                                        "Expected booking event for booking_control action on {event_id}"
                                    );
                                    continue;
                                };
                                cloned_device_controller
                                    .clone()
                                    .control_booking(booking, *action)
                                    .await
                            }
                            TriggerAction::OsdControl(action) => {
                                let Event::Booking { booking } = &event else {
                                    log::warn!(
                                        "Expected booking event for osd_control action on {event_id}"
                                    );
                                    continue;
                                };
                                cloned_device_controller
                                    .clone()
                                    .control_osd(booking, *action)
                                    .await
                            }
                            TriggerAction::BookingReminder => {
                                let Event::Booking { booking } = &event else {
                                    log::warn!(
                                        "Expected booking event for booking_reminder action on {event_id}"
                                    );
                                    continue;
                                };
                                cloned_device_controller
                                    .clone()
                                    .send_booking_reminder(booking)
                                    .await
                            }
                        };

                        if let Err(e) = result {
                            log::error!("Could not trigger action: {e}");
                        }
                    }
                }
            });
            arc_self.handlers.lock().push(task);
        }

        arc_self
    }
}
