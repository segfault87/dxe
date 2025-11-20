use actix_web::web;
use chrono::Utc;
use dxe_data::queries::booking::{get_booking_with_user_id, get_cash_payment_status};
use dxe_data::types::BookingId;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::models::entities::{Booking, BookingCashPaymentStatus};
use crate::models::handlers::booking::GetBookingResponse;
use crate::models::{Error, IntoView};
use crate::session::UserSession;

pub async fn get(
    session: UserSession,
    booking_id: web::Path<BookingId>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetBookingResponse>, Error> {
    let now = Utc::now();

    let mut tx = database.begin().await?;

    let booking = get_booking_with_user_id(&mut *tx, booking_id.as_ref(), &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    let cash_payment_status = get_cash_payment_status(&mut *tx, booking_id.as_ref()).await?;

    Ok(web::Json(GetBookingResponse {
        booking: Booking::convert(booking, &timezone_config, &now)
            .finish(booking_config.as_ref(), &now),
        cash_payment_status: cash_payment_status
            .map(|v| BookingCashPaymentStatus::convert(v, &timezone_config, &now)),
    }))
}
