use actix_web::web;
use chrono::{NaiveTime, TimeDelta};
use dxe_data::queries::booking::get_occupied_slots;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::OccupiedSlot;
use crate::models::handlers::booking::{CalendarQuery, CalendarResponse};
use crate::models::{Error, IntoView};
use crate::session::UserSession;

pub async fn get(
    now: Now,
    _session: UserSession,
    query: web::Query<CalendarQuery>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
    booking_config: web::Data<BookingConfig>,
) -> Result<web::Json<CalendarResponse>, Error> {
    let start = timezone_config
        .convert(*now)
        .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap()
        .to_utc();
    let end = start + TimeDelta::days(booking_config.lookahead_days);

    let mut connection = database.acquire().await?;

    let mut slots = get_occupied_slots(&mut connection, &now, &query.unit_id, &start, &end).await?;
    slots.sort_by(|a, b| a.time_from.cmp(&b.time_from));

    Ok(web::Json(CalendarResponse {
        start: timezone_config.convert(start),
        end: timezone_config.convert(end),
        max_booking_hours: booking_config.max_booking_hours,
        slots: slots
            .into_iter()
            .map(|v| OccupiedSlot::convert(v, &timezone_config, &now))
            .collect::<Result<_, _>>()?,
    }))
}
