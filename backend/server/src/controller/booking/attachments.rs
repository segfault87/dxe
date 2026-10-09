use actix_web::web;
use chrono::TimeDelta;
use dxe_data::queries::booking::{get_booking_attachments, is_booking_attachment_available};
use sqlx::SqlitePool;

use crate::config::TimeZoneConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::entities::BookingAttachment;
use crate::models::handlers::booking::{GetBookingAttachmentsQuery, GetBookingAttachmentsResponse};
use crate::models::{Error, IntoView};
use crate::session::UserSession;
use crate::utils::datetime::truncate_time;

pub async fn get(
    now: Now,
    _session: UserSession,
    query: web::Query<GetBookingAttachmentsQuery>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetBookingAttachmentsResponse>, Error> {
    let mut connection = database.acquire().await?;

    let attachments = get_booking_attachments(&mut connection, &now).await?;

    // Availability is only reported when a time range is given
    let mut unavailable_attachment_ids = Vec::new();
    if let Some(time_from) = query.time_from
        && let Some(desired_hours) = query.desired_hours
    {
        if desired_hours <= 0 {
            return Err(Error::InvalidTimeRange);
        }

        let time_from = truncate_time(time_from).to_utc();
        let time_to = time_from + TimeDelta::hours(desired_hours);

        for attachment in &attachments {
            if !is_booking_attachment_available(
                &mut connection,
                &now,
                &attachment.id,
                &time_from,
                &time_to,
                query.exclude_booking_id.as_ref(),
                query.exclude_adhoc_reservation_id.as_ref(),
            )
            .await?
            {
                unavailable_attachment_ids.push(attachment.id);
            }
        }
    }

    Ok(web::Json(GetBookingAttachmentsResponse {
        attachments: attachments
            .into_iter()
            .map(|v| BookingAttachment::convert(v, &timezone_config, &now))
            .collect::<Result<_, _>>()?,
        unavailable_attachment_ids,
    }))
}
