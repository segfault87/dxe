use chrono::{DateTime, Utc};
use dxe_data::entities;

use super::{
    AdhocParking, AdhocReservation, AudioRecording, Booking, BookingStatus, CashTransaction, Group,
    GroupWithUsers, Identity, OccupiedSlot, SelfUser, TelemetryEntry, TelemetryType,
    TossPaymentsTransaction, User,
};
use crate::config::{BookingConfig, TimeZoneConfig};
use crate::models::Error;
use crate::utils::datetime::is_in_effect;
use crate::utils::mask_identity;

pub trait IntoView: Sized {
    type Entity;
    type Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error>;
}

impl IntoView for User {
    type Entity = entities::User;
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        _now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: entity.id,
            name: entity.name,
            created_at: timezone.convert(entity.created_at),
        })
    }
}

impl IntoView for SelfUser {
    type Entity = entities::User;
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        _now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: entity.id,
            name: entity.name,
            license_plate_number: entity.license_plate_number,
            created_at: timezone.convert(entity.created_at),
            is_administrator: false,
            depositor_name: None,
            refund_account: None,
        })
    }
}

impl IntoView for Group {
    type Entity = entities::Group;
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        _now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: entity.id,
            name: entity.name,
            owner_id: entity.owner_id,
            is_open: entity.is_open,
            created_at: timezone.convert(entity.created_at),
        })
    }
}

impl IntoView for GroupWithUsers {
    type Entity = (entities::Group, Vec<entities::User>);
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        let users: Result<Vec<User>, _> = entity
            .1
            .into_iter()
            .map(|v| User::convert(v, timezone, now))
            .collect();
        let mut users = users?;
        users.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(Self {
            id: entity.0.id,
            name: entity.0.name,
            owner_id: entity.0.owner_id,
            is_open: entity.0.is_open,
            created_at: timezone.convert(entity.0.created_at),
            users,
        })
    }
}

impl IntoView for Identity {
    type Entity = entities::Identity;
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        Ok(match entity {
            entities::Identity::User(user) => Identity::User(User::convert(user, timezone, now)?),
            entities::Identity::Group(group) => {
                Identity::Group(Group::convert(group, timezone, now)?)
            }
        })
    }
}

impl IntoView for Booking {
    type Entity = entities::Booking;
    type Error = Error;

    fn convert(
        booking: entities::Booking,
        timezone: &TimeZoneConfig,
        now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        let booking_time = booking.time_to - booking.time_from;

        Ok(Self {
            id: booking.id,
            unit_id: booking.unit_id,
            holder: User::convert(booking.holder, timezone, now)?,
            customer: Identity::convert(booking.customer, timezone, now)?,
            booking_start: timezone.convert(booking.time_from),
            booking_end: timezone.convert(booking.time_to),
            booking_hours: booking_time.num_hours(),
            created_at: timezone.convert(booking.created_at),
            confirmed_at: booking.confirmed_at.map(|v| timezone.convert(v)),
            is_confirmed: booking.confirmed_at.map(|v| &v < now).unwrap_or(false),
            canceled_at: booking.canceled_at.map(|v| timezone.convert(v)),
            is_canceled: booking.canceled_at.map(|v| &v < now).unwrap_or(false),
            status: BookingStatus::Pending, // to set later
        })
    }
}

impl Booking {
    pub fn finish(mut self, booking_config: &BookingConfig, now: &DateTime<Utc>) -> Self {
        let start = self.booking_start;
        let end = self.booking_end;

        let start_with_buffer = start - booking_config.buffer_time.0;
        let end_with_buffer = end + booking_config.buffer_time.1;

        self.status = if self.is_canceled {
            BookingStatus::Canceled
        } else if !self.is_confirmed {
            if now > &start {
                BookingStatus::Overdue
            } else {
                BookingStatus::Pending
            }
        } else if &start <= now && now < &end {
            BookingStatus::InProgress
        } else if &start_with_buffer <= now && now < &end_with_buffer {
            BookingStatus::Buffered
        } else if now >= &end_with_buffer {
            BookingStatus::Complete
        } else {
            BookingStatus::Confirmed
        };

        self
    }
}

impl IntoView for OccupiedSlot {
    type Entity = entities::OccupiedSlot;
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        _now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        let booking_time = entity.time_to - entity.time_from;

        Ok(Self {
            masked_name: mask_identity(entity.name),
            booking_date: timezone.convert(entity.time_from),
            booking_hours: booking_time.num_hours(),
            confirmed: entity.confirmed,
        })
    }
}

impl IntoView for CashTransaction {
    type Entity = entities::CashTransaction;
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            depositor_name: entity.depositor_name,
            price: entity.price,
            confirmed_at: entity.confirmed_at.map(|v| timezone.convert(v)),
            refund_price: entity.refund_price,
            refund_account: entity.refund_account,
            is_refund_requested: entity.refund_price.is_some(),
            refunded_at: entity.refunded_at.map(|v| timezone.convert(v)),
            is_refunded: is_in_effect(&entity.refunded_at, now),
        })
    }
}

impl IntoView for TossPaymentsTransaction {
    type Entity = entities::TossPaymentsTransaction;
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            price: entity.price,
            confirmed_at: entity.confirmed_at.map(|v| timezone.convert(v)),
            refund_price: entity.refund_price,
            refunded_at: entity.refunded_at.map(|v| timezone.convert(v)),
            is_refunded: is_in_effect(&entity.refunded_at, now),
        })
    }
}

impl IntoView for AdhocReservation {
    type Entity = entities::AdhocReservation;
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: entity.id,
            holder: User::convert(entity.holder, timezone, now)?,
            customer: Identity::convert(entity.customer, timezone, now)?,
            reservation_start: timezone.convert(entity.time_from),
            reservation_end: timezone.convert(entity.time_to),
            reserved_hours: (entity.time_to - entity.time_from).num_hours(),
            deleted_at: entity.deleted_at.map(|v| timezone.convert(v)),
            remark: entity.remark,
        })
    }
}

impl IntoView for AudioRecording {
    type Entity = entities::AudioRecording;
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        _now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            booking_id: entity.booking_id,
            url: entity
                .url
                .parse()
                .map_err(|e| Error::Internal(Box::new(e)))?,

            created_at: timezone.convert(entity.created_at),
            expires_in: entity.expires_in.map(|v| timezone.convert(v)),
        })
    }
}

impl IntoView for AdhocParking {
    type Entity = entities::AdhocParking;
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        timezone: &TimeZoneConfig,
        _now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: entity.id,
            space_id: entity.space_id,
            time_from: timezone.convert(entity.time_from),
            time_to: timezone.convert(entity.time_to),
            license_plate_number: entity.license_plate_number,
            created_at: timezone.convert(entity.created_at),
        })
    }
}

impl IntoView for TelemetryEntry {
    type Entity = entities::TelemetryFile;
    type Error = Error;

    fn convert(
        entity: Self::Entity,
        _timezone: &TimeZoneConfig,
        _now: &DateTime<Utc>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            r#type: match entity.r#type {
                dxe_types::TelemetryType::PowerUsageRoom => TelemetryType::PowerUsageRoom,
                dxe_types::TelemetryType::PowerUsageTotal => TelemetryType::PowerUsageTotal,
                dxe_types::TelemetryType::SoundMeter => TelemetryType::SoundMeter,
            },
        })
    }
}
