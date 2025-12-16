use actix_web::web;
use dxe_data::queries::booking::{create_audio_recording, get_audio_recording, get_booking};
use dxe_s2s_shared::handlers::UpdateAudioRequest;
use dxe_types::BookingId;
use sqlx::SqlitePool;

use crate::middleware::datetime_injector::Now;
use crate::models::Error;
use crate::services::messaging::MessagingService;

pub async fn post(
    now: Now,
    booking_id: web::Path<BookingId>,
    body: web::Json<UpdateAudioRequest>,
    database: web::Data<SqlitePool>,
    messaging_service: web::Data<MessagingService>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let booking = get_booking(&mut tx, &booking_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    if create_audio_recording(
        &mut tx,
        &now,
        &booking_id,
        body.url.as_str(),
        Some(&body.expires_in.to_utc()),
    )
    .await?
        && let Some(audio_recording) = get_audio_recording(&mut tx, &booking_id).await?
        && let Err(e) = messaging_service
            .send_audio_recording(&mut tx, booking.clone(), audio_recording.clone())
            .await
    {
        log::warn!("Couldn't send audio recording notification: {e}");
    }

    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}
