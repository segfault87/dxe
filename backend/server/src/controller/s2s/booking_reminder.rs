use actix_web::web;
use dxe_data::queries::booking::get_booking;
use dxe_data::utils::is_in_effect;
use dxe_types::BookingId;
use sqlx::SqlitePool;

use crate::middleware::datetime_injector::Now;
use crate::models::Error;
use crate::services::messaging::MessagingService;

pub async fn post(
    now: Now,
    booking_id: web::Path<BookingId>,
    database: web::Data<SqlitePool>,
    messaging_service: web::Data<MessagingService>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let booking = get_booking(&mut tx, &booking_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    if !is_in_effect(&booking.confirmed_at, &now) || is_in_effect(&booking.canceled_at, &now) {
        log::info!("Skipping reminder as the booking is either not confirmed or canceled.");
    } else if *now > booking.time_from {
        log::info!("Skipping reminder as the booking has already started.");
    } else if let Err(e) = messaging_service.send_reminder(&mut tx, booking).await {
        log::warn!("Could not send reminder for booking {booking_id}: {e}");
    }

    Ok(web::Json(serde_json::json!({})))
}
