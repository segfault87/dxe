use std::collections::HashMap;

use actix_web::web;
use chrono::Utc;
use dxe_data::queries::booking::get_bookings_by_user_id;
use dxe_data::queries::user::{get_user_by_id, update_user};
use dxe_types::UnitId;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::models::entities::{Booking, BookingStatus, SelfUser};
use crate::models::handlers::user::{MeResponse, UpdateMeRequest, UpdateMeResponse};
use crate::models::{Error, IntoView};
use crate::session::UserSession;

pub async fn get(
    session: UserSession,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
    booking_config: web::Data<BookingConfig>,
) -> Result<web::Json<MeResponse>, Error> {
    let now = Utc::now();

    let mut connection = database.acquire().await?;

    let user = get_user_by_id(&mut connection, &session.user_id, &now)
        .await?
        .ok_or(Error::UserNotFound)?;

    let user = SelfUser {
        id: user.id,
        name: user.name,
        license_plate_number: user.license_plate_number,
        created_at: timezone_config.convert(user.created_at),
        is_administrator: session.is_administrator,
    };

    let mut bookings =
        get_bookings_by_user_id(&mut connection, &now, &session.user_id, &now, false).await?;
    bookings.sort_by(|a, b| a.time_from.cmp(&b.time_from));

    let mut active_bookings = HashMap::new();
    let mut pending_bookings: HashMap<UnitId, Vec<Booking>> = HashMap::new();

    for booking in bookings {
        let unit_id = booking.unit_id.clone();

        let booking =
            Booking::convert(booking, &timezone_config, &now).finish(booking_config.as_ref(), &now);

        if !active_bookings.contains_key(&unit_id) && booking.status == BookingStatus::Buffered
            || booking.status == BookingStatus::InProgress
        {
            active_bookings.insert(unit_id, booking);
        } else if booking.status == BookingStatus::Confirmed
            || booking.status == BookingStatus::Pending
        {
            pending_bookings.entry(unit_id).or_default().push(booking);
        }
    }

    Ok(web::Json(MeResponse {
        user,
        active_bookings,
        pending_bookings,
    }))
}

pub async fn post(
    session: UserSession,
    body: web::Json<UpdateMeRequest>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<UpdateMeResponse>, Error> {
    let now = Utc::now();

    let mut tr = database.begin().await?;

    let result = update_user(
        &mut tr,
        &now,
        &session.user_id,
        &body.new_name,
        &body.new_license_plate_number,
    )
    .await?;

    tr.commit().await?;

    let user = SelfUser {
        id: result.id,
        name: result.name,
        license_plate_number: result.license_plate_number,
        created_at: timezone_config.convert(result.created_at),
        is_administrator: session.is_administrator,
    };

    Ok(web::Json(UpdateMeResponse { user }))
}
