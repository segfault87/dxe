use actix_web::web;
use chrono::TimeDelta;
use dxe_data::queries::booking::is_booking_available;
use dxe_data::queries::unit::is_unit_enabled;
use sqlx::SqlitePool;

use crate::config::BookingConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::Error;
use crate::models::handlers::booking::{CheckRequest, CheckResponse};
use crate::session::UserSession;
use crate::utils::datetime::truncate_time;

pub async fn post(
    now: Now,
    _session: UserSession,
    body: web::Json<CheckRequest>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
) -> Result<web::Json<CheckResponse>, Error> {
    let mut connection = database.acquire().await?;

    if is_unit_enabled(&mut connection, &body.unit_id).await? != Some(true) {
        return Err(Error::UnitNotFound);
    }

    if body.desired_hours > booking_config.max_booking_hours {
        return Err(Error::InvalidTimeRange);
    }

    let time_from = truncate_time(body.time_from).to_utc();
    let time_to = time_from + TimeDelta::hours(body.desired_hours);

    if is_booking_available(
        &mut connection,
        &now,
        &body.unit_id,
        &time_from,
        &time_to,
        body.exclude_booking_id.as_ref(),
        body.exclude_adhoc_reservation_id.as_ref(),
    )
    .await?
    {
        let total_price = if let Some(additional_hours) = body.additional_hours {
            booking_config
                .calculate_additive_price(&body.unit_id, additional_hours)
                .map_err(|_| Error::UnitNotFound)?
        } else {
            booking_config
                .calculate_price(&body.unit_id, time_from, time_to)
                .map_err(|_| Error::UnitNotFound)?
        };

        Ok(web::Json(CheckResponse { total_price }))
    } else {
        Err(Error::TimeRangeOccupied)
    }
}
