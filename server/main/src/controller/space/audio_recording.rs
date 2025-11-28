use actix_web::web;
use chrono::Utc;
use dxe_data::queries::booking::{create_audio_recording, get_booking};
use dxe_s2s_shared::handlers::UpdateAudioRequest;
use dxe_types::BookingId;
use sqlx::SqlitePool;

use crate::models::Error;
use crate::services::messaging::biztalk::BiztalkSender;

pub async fn post(
    booking_id: web::Query<BookingId>,
    body: web::Json<UpdateAudioRequest>,
    database: web::Data<SqlitePool>,
    biztalk_sender: web::Data<Option<BiztalkSender>>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let now = Utc::now();

    let mut tx = database.begin().await?;

    let booking = get_booking(&mut tx, &booking_id).await?.ok_or(Error::BookingNotFound)?;

    if create_audio_recording(
        &mut tx,
        &now,
        &booking_id,
        body.url.as_str(),
        Some(&body.expires_in.to_utc()),
    )
    .await?
    {
        let users = get_bo
    }

    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}
