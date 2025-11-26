use std::collections::HashMap;

use actix_web::web;
use chrono::{TimeDelta, Utc};
use dxe_data::entities::{Booking as RawBooking, Identity, User as RawUser};
use dxe_data::queries::identity::get_group_members;
use dxe_data::queries::{booking::get_bookings_by_unit_id, unit::get_units_by_space_id};
use dxe_s2s_shared::entities::{Booking, BookingWithUsers, User};
use dxe_s2s_shared::handlers::{BookingType, GetBookingsQuery, GetBookingsResponse};
use sqlx::SqlitePool;

use crate::config::BookingConfig;
use crate::middleware::coordinator_verifier::CoordinatorContext;
use crate::models::Error;

fn convert_user(user: &RawUser) -> User {
    User {
        id: user.id.clone(),
        name: user.name.clone(),
        license_plate_number: user.license_plate_number.clone(),
    }
}

fn convert_booking(booking: &RawBooking, booking_config: &BookingConfig) -> Booking {
    Booking {
        id: booking.id.clone(),
        unit_id: booking.unit_id.clone(),
        date_start_w_buffer: (booking.time_from - booking_config.buffer_time.0).into(),
        date_start: booking.time_from.into(),
        date_end: booking.time_to.into(),
        date_end_w_buffer: (booking.time_to + booking_config.buffer_time.1).into(),
        customer_id: booking.customer.id(),
        customer_name: booking.customer.name().to_owned(),
    }
}

pub async fn get(
    context: CoordinatorContext,
    query: web::Query<GetBookingsQuery>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
) -> Result<web::Json<GetBookingsResponse>, Error> {
    let now = Utc::now();

    let CoordinatorContext { space_id } = context;

    let start = now - booking_config.buffer_time.0;
    let end = now + TimeDelta::days(1) + booking_config.buffer_time.1;

    let mut tx = database.begin().await?;

    let units = get_units_by_space_id(&mut tx, &space_id).await?;
    let mut result = HashMap::new();

    let (confirmed_only, exclude_canceled) = match query.r#type {
        BookingType::All => (false, false),
        BookingType::Confirmed => (true, true),
        BookingType::Pending => (false, true),
    };

    for unit in units {
        let mut bookings = vec![];
        for booking in get_bookings_by_unit_id(
            &mut tx,
            &now,
            &unit.id,
            &start,
            &end,
            confirmed_only,
            exclude_canceled,
        )
        .await?
        {
            let users = match &booking.customer {
                Identity::User(u) => vec![convert_user(u)],
                Identity::Group(g) => get_group_members(&mut tx, &g.id)
                    .await?
                    .iter()
                    .map(convert_user)
                    .collect(),
            };

            bookings.push(BookingWithUsers {
                booking: convert_booking(&booking, &booking_config),
                users,
            });
        }

        result.insert(unit.id.clone(), bookings);
    }

    Ok(web::Json(GetBookingsResponse {
        range_start: start.into(),
        range_end: end.into(),
        bookings: result,
    }))
}
