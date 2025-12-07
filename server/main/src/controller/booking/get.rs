use actix_web::web;
use dxe_data::queries::booking::{
    get_booking_with_user_id, get_cash_payment_status, get_toss_payment_status_by_booking_id,
};
use dxe_types::BookingId;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{Booking, BookingCashPaymentStatus, BookingTossPaymentStatus};
use crate::models::handlers::booking::GetBookingResponse;
use crate::models::{Error, IntoView};
use crate::session::UserSession;

pub async fn get(
    now: Now,
    session: UserSession,
    booking_id: web::Path<BookingId>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetBookingResponse>, Error> {
    let mut tx = database.begin().await?;

    let booking = get_booking_with_user_id(&mut tx, booking_id.as_ref(), &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    let cash_payment_status = get_cash_payment_status(&mut tx, &booking_id).await?;
    let toss_payment_status = get_toss_payment_status_by_booking_id(&mut tx, &booking_id).await?;

    Ok(web::Json(GetBookingResponse {
        booking: Booking::convert(booking, &timezone_config, &now)?
            .finish(booking_config.as_ref(), &now),
        cash_payment_status: if let Some(cash_payment_status) = cash_payment_status {
            Some(BookingCashPaymentStatus::convert(
                cash_payment_status,
                &timezone_config,
                &now,
            )?)
        } else {
            None
        },
        toss_payment_status: if let Some(toss_payment_status) = toss_payment_status {
            Some(BookingTossPaymentStatus::convert(
                toss_payment_status,
                &timezone_config,
                &now,
            )?)
        } else {
            None
        },
    }))
}
