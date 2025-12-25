use actix_web::web;
use dxe_extern::itsokey::Error as ItsokeyError;

use crate::middleware::coordinator_verifier::CoordinatorContext;
use crate::models::Error;
use crate::services::doorlock::{DoorLockService, Error as DoorLockError};
use crate::services::notification::{NotificationSender, Priority};

pub async fn post(
    context: CoordinatorContext,
    doorlock_service: web::Data<DoorLockService>,
    notification_sender: web::Data<NotificationSender>,
) -> Result<web::Json<serde_json::Value>, Error> {
    doorlock_service
        .open(&context.space_id)
        .await
        .map_err(|e| {
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
        format!("Space {} opened the door", context.space_id),
    );

    Ok(web::Json(serde_json::json!({})))
}
