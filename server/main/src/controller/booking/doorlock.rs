use actix_web::web;
use chrono::Utc;
use dxe_data::queries::booking::get_booking_with_user_id;
use dxe_data::queries::user::get_user_by_id;
use dxe_data::types::BookingId;
use dxe_extern::itsokey::Error as ItsokeyError;
use sqlx::SqlitePool;

use crate::config::BookingConfig;
use crate::models::Error;
use crate::services::doorlock::{DoorLockService, Error as DoorLockError};
use crate::services::telemetry::{NotificationSender, Priority};
use crate::session::UserSession;

pub async fn post(
    session: UserSession,
    booking_id: web::Path<BookingId>,
    database: web::Data<SqlitePool>,
    doorlock_service: web::Data<DoorLockService>,
    booking_config: web::Data<BookingConfig>,
    notification_sender: web::Data<NotificationSender>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let now = Utc::now();

    let mut connection = database.acquire().await?;

    let booking = get_booking_with_user_id(&mut connection, booking_id.as_ref(), &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    if !booking_config.is_in_buffer(&now, booking.time_from, booking.time_to) {
        return Err(Error::BookingNotActive);
    }

    let user = get_user_by_id(&mut connection, &session.user_id, &now)
        .await?
        .ok_or(Error::UserNotFound)?;

    doorlock_service.open().await.map_err(|e| {
        notification_sender.enqueue(Priority::High, format!("Could not open the door: {e}"));

        match e {
            DoorLockError::Itsokey(e) => match e {
                ItsokeyError::Http(_) => {
                    Error::DoorNotOpened("Itsokey 서버와 통신에 실패했습니다.".to_owned())
                }
                ItsokeyError::Json(_) => Error::DoorNotOpened(
                    "Itsokey 서버에서 내려온 응답 해독에 실패했습니다.".to_owned(),
                ),
                ItsokeyError::Itsokey(e) => Error::DoorNotOpened(e),
            },
        }
    })?;

    notification_sender.enqueue(
        Priority::Default,
        format!("User {} opened the door", user.name),
    );

    Ok(web::Json(serde_json::json!({})))
}
