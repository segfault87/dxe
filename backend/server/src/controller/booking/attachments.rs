use actix_web::web;
use dxe_data::queries::booking::get_booking_attachments;
use sqlx::SqlitePool;

use crate::config::TimeZoneConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::entities::BookingAttachment;
use crate::models::handlers::booking::GetBookingAttachmentsResponse;
use crate::models::{Error, IntoView};
use crate::session::UserSession;

pub async fn get(
    now: Now,
    _session: UserSession,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetBookingAttachmentsResponse>, Error> {
    let mut connection = database.acquire().await?;

    let attachments = get_booking_attachments(&mut connection, &now).await?;

    Ok(web::Json(GetBookingAttachmentsResponse {
        attachments: attachments
            .into_iter()
            .map(|v| BookingAttachment::convert(v, &timezone_config, &now))
            .collect::<Result<_, _>>()?,
    }))
}
