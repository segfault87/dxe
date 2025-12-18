use actix_web::web;
use dxe_data::queries::booking::{expire_adhoc_reservation, get_adhoc_reservation};
use dxe_types::AdhocReservationId;
use sqlx::SqlitePool;

use crate::middleware::datetime_injector::Now;
use crate::models::Error;
use crate::session::UserSession;

pub async fn delete(
    now: Now,
    user: UserSession,
    adhoc_reservation_id: web::Path<AdhocReservationId>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let adhoc_reservation = get_adhoc_reservation(&mut tx, adhoc_reservation_id.as_ref())
        .await?
        .ok_or(Error::BookingNotFound)?;

    if adhoc_reservation.holder.id != user.user_id {
        return Err(Error::BookingNotFound);
    }

    expire_adhoc_reservation(&mut tx, &now, adhoc_reservation_id.as_ref()).await?;

    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}
