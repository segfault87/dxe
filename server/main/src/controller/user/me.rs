use std::collections::HashMap;

use actix_web::web;
use dxe_data::queries::booking::get_bookings_by_user_id;
use dxe_data::queries::user::{get_user_by_id, get_user_cash_payment_information, update_user};
use dxe_types::UnitId;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig, UrlConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{Booking, BookingStatus, SelfUser};
use crate::models::handlers::user::{MeResponse, UpdateMeRequest, UpdateMeResponse};
use crate::models::{Error, IntoView};
use crate::session::UserSession;

pub async fn get(
    now: Now,
    session: UserSession,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
    booking_config: web::Data<BookingConfig>,
    url_config: web::Data<UrlConfig>,
) -> Result<web::Json<MeResponse>, Error> {
    let mut tx = database.begin().await?;

    let user = get_user_by_id(&mut tx, &session.user_id, &now)
        .await?
        .ok_or(Error::LoggedOut(url_config.clone()))?;

    let cash_payment_information =
        get_user_cash_payment_information(&mut tx, &session.user_id).await?;

    let user = SelfUser {
        id: user.id,
        name: user.name,
        license_plate_number: user.license_plate_number,
        created_at: timezone_config.convert(user.created_at),
        is_administrator: session.is_administrator,
        depositor_name: cash_payment_information
            .as_ref()
            .and_then(|v| v.depositor_name.clone()),
        refund_account: cash_payment_information
            .as_ref()
            .and_then(|v| v.refund_account.clone()),
        use_pg_payment: user.use_pg_payment,
    };

    let mut bookings =
        get_bookings_by_user_id(&mut tx, &now, &session.user_id, &now, false).await?;
    bookings.sort_by(|a, b| a.time_from.cmp(&b.time_from));

    let mut active_bookings = HashMap::new();
    let mut pending_bookings: HashMap<UnitId, Vec<Booking>> = HashMap::new();

    for booking in bookings {
        let unit_id = booking.unit_id.clone();

        let booking = Booking::convert(booking, &timezone_config, &now)?
            .finish(booking_config.as_ref(), &now);

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
    now: Now,
    session: UserSession,
    body: web::Json<UpdateMeRequest>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<UpdateMeResponse>, Error> {
    let mut tx = database.begin().await?;

    let result = update_user(
        &mut tx,
        &now,
        &session.user_id,
        &body.new_name,
        &body.new_license_plate_number,
    )
    .await?;

    let cash_payment_information =
        get_user_cash_payment_information(&mut tx, &session.user_id).await?;

    tx.commit().await?;

    let user = SelfUser {
        id: result.id,
        name: result.name,
        license_plate_number: result.license_plate_number,
        created_at: timezone_config.convert(result.created_at),
        is_administrator: session.is_administrator,
        depositor_name: cash_payment_information
            .as_ref()
            .and_then(|v| v.depositor_name.clone()),
        refund_account: cash_payment_information
            .as_ref()
            .and_then(|v| v.refund_account.clone()),
        use_pg_payment: result.use_pg_payment,
    };

    Ok(web::Json(UpdateMeResponse { user }))
}
