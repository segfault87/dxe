use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

use chrono::TimeDelta;
use dxe_s2s_shared::entities::BookingWithUsers;
use dxe_types::UnitId;
use futures::Stream;
use parking_lot::Mutex;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

use crate::types::{EventId, PresenceEvent, TenantId};

static DEFAULT_EVENT_CHANNEL_CAPACITY: usize = 100;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Event {
    Booking {
        booking: BookingWithUsers,
    },
    Alert {
        unit_ids: Option<HashSet<UnitId>>,
        grace: Option<TimeDelta>,
        debounce: Option<TimeDelta>,
    },
    Presence {
        tenant_id: TenantId,
        r#type: PresenceEvent,
    },
}

pub struct EventReceiver {
    subscribed_events: HashSet<EventId>,
    receiver: BroadcastStream<(EventId, Event)>,
}

impl Stream for EventReceiver {
    type Item = Result<(EventId, Event), BroadcastStreamRecvError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let inner = Pin::new(&mut this.receiver);
        match inner.poll_next(cx) {
            Poll::Ready(Some(Ok((id, value)))) => {
                if this.subscribed_events.contains(&id) {
                    Poll::Ready(Some(Ok((id, value))))
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            other => other,
        }
    }
}

#[derive(Clone)]
pub struct EventSender {
    broadcasts: Arc<Mutex<broadcast::Sender<(EventId, Event)>>>,
}

impl EventSender {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(DEFAULT_EVENT_CHANNEL_CAPACITY);

        Self {
            broadcasts: Arc::new(Mutex::new(sender)),
        }
    }

    pub fn subscribe(&self, event_ids: impl Iterator<Item = EventId>) -> EventReceiver {
        let guard = self.broadcasts.lock();
        let receiver = guard.subscribe();

        EventReceiver {
            subscribed_events: event_ids.collect(),
            receiver: BroadcastStream::new(receiver),
        }
    }

    pub fn publish(&self, event_id: EventId, event: Event) {
        let _ = self.broadcasts.lock().send((event_id, event));
    }
}
