use std::collections::HashMap;
use std::sync::Arc;

use dxe_data::entities::{Booking, Reservation, User};
use dxe_extern::google_cloud::calendar::{
    DateTimeRepresentation, Error as CalendarError, Event, EventId, EventVisibility,
    ExtendedProperties, GoogleCalendarClient,
};
use dxe_extern::google_cloud::{CredentialManager, Error as GcpError};
use dxe_types::{BookingId, ReservationId};

use crate::config::{GoogleApiConfig, TimeZoneConfig};

#[derive(Clone, Debug)]
pub struct CalendarService {
    client: Arc<GoogleCalendarClient>,
}

fn booking_id_to_event_id(booking_id: &BookingId) -> EventId {
    booking_id.to_string().replace("-", "").into()
}

fn adhoc_reservation_id_to_event_id(reservation_id: &ReservationId) -> EventId {
    format!("adhoc{reservation_id}").into()
}

impl CalendarService {
    pub fn new(config: &GoogleApiConfig) -> Result<Self, Error> {
        let credential = CredentialManager::new(config, Some(config.calendar.identity.clone()))?;
        let client = GoogleCalendarClient::new(credential, &config.calendar);

        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub async fn register_booking(
        &self,
        timezone_config: &TimeZoneConfig,
        booking: &Booking,
        users: &Vec<User>,
    ) -> Result<(), Error> {
        let event = Event {
            id: booking_id_to_event_id(&booking.id),
            summary: booking.customer.name().to_owned(),
            description: Default::default(),
            start: DateTimeRepresentation {
                date_time: timezone_config.convert(booking.time_from),
            },
            end: DateTimeRepresentation {
                date_time: timezone_config.convert(booking.time_to),
            },
            visibility: EventVisibility::Private,
            extended_properties: ExtendedProperties {
                private: HashMap::from([
                    ("event_type".to_owned(), "booking".to_owned()),
                    ("booking_id".to_owned(), booking.id.to_string()),
                    (
                        "license_plate_numbers".to_owned(),
                        users
                            .iter()
                            .filter_map(|v| {
                                v.license_plate_number.as_ref().and_then(|v| {
                                    if v.is_empty() {
                                        None
                                    } else {
                                        Some(v.to_owned())
                                    }
                                })
                            })
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    (
                        "customers".to_owned(),
                        users
                            .iter()
                            .map(|v| v.name.clone())
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                ]),
                ..Default::default()
            },
        };

        Ok(self.client.create_event(event).await?)
    }

    pub async fn register_adhoc_reservation(
        &self,
        adhoc_reservation: &Reservation,
        timezone_config: &TimeZoneConfig,
    ) -> Result<(), Error> {
        let event = Event {
            id: adhoc_reservation_id_to_event_id(&adhoc_reservation.id),
            summary: adhoc_reservation.holder.name.clone(),
            description: Default::default(),
            start: DateTimeRepresentation {
                date_time: timezone_config.convert(adhoc_reservation.time_from),
            },
            end: DateTimeRepresentation {
                date_time: timezone_config.convert(adhoc_reservation.time_to),
            },
            visibility: EventVisibility::Private,
            extended_properties: ExtendedProperties {
                private: HashMap::from([("event_type".to_owned(), "adhoc_reservation".to_owned())]),
                ..Default::default()
            },
        };

        Ok(self.client.create_event(event).await?)
    }

    pub async fn delete_booking(&self, booking_id: &BookingId) -> Result<(), Error> {
        Ok(self.client.delete_event(&booking_id.to_string()).await?)
    }

    pub async fn delete_adhoc_reservation(
        &self,
        adhoc_reservation_id: &ReservationId,
    ) -> Result<(), Error> {
        Ok(self
            .client
            .delete_event(&format!("adhoc{adhoc_reservation_id}"))
            .await?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("GCP authentication error: {0}")]
    Gcp(#[from] GcpError),
    #[error("Google Calendar error: {0}")]
    GoogleCalendar(#[from] CalendarError),
}
