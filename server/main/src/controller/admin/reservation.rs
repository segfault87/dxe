use actix_web::web;
use chrono::{TimeDelta, Utc};
use dxe_data::queries::booking::{
    create_reservation, delete_reservation, get_reservation, get_reservations_by_unit_id,
    is_booking_available,
};
use dxe_types::ReservationId;
use sqlx::SqlitePool;

use crate::config::TimeZoneConfig;
use crate::models::entities::Reservation;
use crate::models::handlers::admin::{
    CreateReservationRequest, CreateReservationResponse, GetReservationsQuery,
    GetReservationsResponse,
};
use crate::models::{Error, IntoView};
use crate::session::UserSession;
use crate::utils::datetime::truncate_time;

pub async fn get(
    query: web::Query<GetReservationsQuery>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetReservationsResponse>, Error> {
    let now = Utc::now();

    let mut connection = database.acquire().await?;

    let reservations =
        get_reservations_by_unit_id(&mut connection, &query.unit_id, Some(now)).await?;

    Ok(web::Json(GetReservationsResponse {
        reservations: reservations
            .into_iter()
            .map(|v| Reservation::convert(v, &timezone_config, &now))
            .collect::<Result<_, _>>()?,
    }))
}

pub async fn post(
    session: UserSession,
    body: web::Json<CreateReservationRequest>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<CreateReservationResponse>, Error> {
    let now = Utc::now();

    let mut tx = database.begin().await?;

    let time_from = truncate_time(body.time_from).to_utc();
    let time_to = time_from + TimeDelta::hours(body.desired_hours);

    if !is_booking_available(&mut tx, &now, &body.unit_id, &time_from, &time_to).await? {
        return Err(Error::TimeRangeOccupied);
    }

    let id = create_reservation(
        &mut tx,
        &now,
        &body.unit_id,
        &session.user_id,
        &time_from,
        &time_to,
        &body.remark,
        body.temporary,
    )
    .await?;

    let reservation = get_reservation(&mut tx, id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    tx.commit().await?;

    Ok(web::Json(CreateReservationResponse {
        reservation: Reservation::convert(reservation, &timezone_config, &now)?,
    }))
}

pub async fn delete(
    path: web::Path<ReservationId>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let reservation = get_reservation(&mut tx, path.into_inner())
        .await?
        .ok_or(Error::BookingNotFound)?;

    delete_reservation(&mut tx, reservation.id).await?;

    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}
