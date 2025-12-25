use actix_web::web;
use dxe_data::queries::booking::get_booking_with_user_id;
use dxe_data::queries::unit::get_space_by_unit_id;
use dxe_data::queries::user::get_user_by_id;
use dxe_extern::itsokey::Error as ItsokeyError;
use dxe_types::BookingId;
use sqlx::SqlitePool;

use crate::config::BookingConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::Error;
use crate::services::doorlock::{DoorLockService, Error as DoorLockError};
use crate::services::notification::{NotificationSender, Priority};
use crate::session::UserSession;

pub async fn post(
    now: Now,
    session: UserSession,
    booking_id: web::Path<BookingId>,
    database: web::Data<SqlitePool>,
    doorlock_service: web::Data<DoorLockService>,
    booking_config: web::Data<BookingConfig>,
    notification_sender: web::Data<NotificationSender>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let booking = get_booking_with_user_id(&mut tx, booking_id.as_ref(), &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    if !booking_config.is_in_buffer(&now, booking.time_from, booking.time_to) {
        return Err(Error::BookingNotActive);
    }

    let user = get_user_by_id(&mut tx, &session.user_id, &now)
        .await?
        .ok_or(Error::UserNotFound)?;

    let space = get_space_by_unit_id(&mut tx, &booking.unit_id)
        .await?
        .ok_or(Error::UnitNotFound)?;

    doorlock_service.open(&space.id).await.map_err(|e| {
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
            DoorLockError::NoConfiguration(_) => {
                Error::DoorNotOpened("서버에 오류가 발생했씁니다. 문의 바랍니다.".to_owned())
            }
        }
    })?;

    notification_sender.enqueue(
        Priority::Default,
        format!("User {} opened the door", user.name),
    );

    Ok(web::Json(serde_json::json!({})))
}
