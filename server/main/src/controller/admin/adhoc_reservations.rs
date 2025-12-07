use actix_web::web;
use chrono::{DateTime, TimeDelta};
use dxe_data::queries::booking::{
    create_adhoc_reservation, delete_adhoc_reservation, get_adhoc_reservation,
    get_adhoc_reservations_by_unit_id, is_booking_available,
};
use dxe_types::AdhocReservationId;
use sqlx::SqlitePool;

use crate::config::TimeZoneConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::entities::AdhocReservation;
use crate::models::handlers::admin::{
    CreateAdhocReservationRequest, CreateAdhocReservationResponse, GetAdhocReservationsQuery,
    GetAdhocReservationsResponse,
};
use crate::models::{Error, IntoView};
use crate::services::calendar::CalendarService;
use crate::session::UserSession;
use crate::utils::datetime::truncate_time;

pub async fn get(
    now: Now,
    query: web::Query<GetAdhocReservationsQuery>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetAdhocReservationsResponse>, Error> {
    let mut connection = database.acquire().await?;

    let reservations =
        get_adhoc_reservations_by_unit_id(&mut connection, &now, &query.unit_id, Some(*now))
            .await?;

    Ok(web::Json(GetAdhocReservationsResponse {
        reservations: reservations
            .into_iter()
            .map(|v| AdhocReservation::convert(v, &timezone_config, &now))
            .collect::<Result<_, _>>()?,
    }))
}

pub async fn post(
    now: Now,
    session: UserSession,
    body: web::Json<CreateAdhocReservationRequest>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
    calendar_service: web::Data<Option<CalendarService>>,
) -> Result<web::Json<CreateAdhocReservationResponse>, Error> {
    let mut tx = database.begin().await?;

    let time_from = truncate_time(body.time_from).to_utc();
    let time_to = time_from + TimeDelta::hours(body.desired_hours);

    let expires_at = body.expires_at.as_ref().map(DateTime::to_utc);

    if !is_booking_available(&mut tx, &now, &body.unit_id, &time_from, &time_to).await? {
        return Err(Error::TimeRangeOccupied);
    }

    let id = create_adhoc_reservation(
        &mut tx,
        &now,
        &body.unit_id,
        &body.customer_id,
        &session.user_id,
        &time_from,
        &time_to,
        &body.remark,
        &expires_at,
    )
    .await?;

    let reservation = get_adhoc_reservation(&mut tx, &id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    tx.commit().await?;

    if let Some(calendar_service) = calendar_service.as_ref() {
        if let Err(e) = calendar_service
            .register_adhoc_reservation(&reservation, &timezone_config)
            .await
        {
            log::error!("Failed to create ad hoc reservation on calendar: {e}");
        }
    }

    Ok(web::Json(CreateAdhocReservationResponse {
        reservation: AdhocReservation::convert(reservation, &timezone_config, &now)?,
    }))
}

pub async fn delete(
    reservation_id: web::Path<AdhocReservationId>,
    database: web::Data<SqlitePool>,
    calendar_service: web::Data<Option<CalendarService>>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let reservation = get_adhoc_reservation(&mut tx, &reservation_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    delete_adhoc_reservation(&mut tx, reservation.id).await?;

    tx.commit().await?;

    if let Some(calendar_service) = calendar_service.as_ref() {
        if let Err(e) = calendar_service
            .delete_adhoc_reservation(&reservation_id)
            .await
        {
            log::error!("Failed to delete ad hoc reservation on calendar: {e}");
        }
    }

    Ok(web::Json(serde_json::json!({})))
}
