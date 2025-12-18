use actix_web::web;
use dxe_data::queries::booking::{get_audio_recording, get_booking_with_user_id};
use dxe_types::BookingId;
use sqlx::SqlitePool;

use crate::config::TimeZoneConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::entities::AudioRecording;
use crate::models::handlers::booking::GetAudioRecordingResponse;
use crate::models::{Error, IntoView};
use crate::session::UserSession;
use crate::utils::datetime::is_in_effect;

pub async fn get(
    now: Now,
    session: UserSession,
    booking_id: web::Path<BookingId>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetAudioRecordingResponse>, Error> {
    let mut tx = database.begin().await?;

    let booking = get_booking_with_user_id(&mut tx, &booking_id, &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    let audio_recording = get_audio_recording(&mut tx, &booking.id)
        .await?
        .ok_or(Error::AudioRecordingNotFound)?;

    let response = GetAudioRecordingResponse {
        audio_recording: if is_in_effect(&audio_recording.expires_in, &now) {
            None
        } else {
            Some(AudioRecording::convert(
                audio_recording,
                &timezone_config,
                &now,
            )?)
        },
    };

    Ok(web::Json(response))
}
