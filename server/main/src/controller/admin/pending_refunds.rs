use actix_web::web;
use chrono::Utc;
use dxe_data::queries::booking::get_bookings_refund_pending;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::models::entities::{Booking, BookingCashPaymentStatus, BookingWithPayments};
use crate::models::handlers::admin::GetRefundPendingBookingsResponse;
use crate::models::{Error, IntoView};
use crate::session::UserSession;

pub async fn get(
    _session: UserSession,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetRefundPendingBookingsResponse>, Error> {
    let now = Utc::now();

    let mut connection = database.acquire().await?;

    let bookings = get_bookings_refund_pending(&mut connection).await?;

    Ok(web::Json(GetRefundPendingBookingsResponse {
        bookings: bookings
            .into_iter()
            .map(|(booking, payment)| BookingWithPayments {
                booking: Booking::convert(booking, &timezone_config, &now)
                    .finish(&booking_config, &now),
                payment: Some(BookingCashPaymentStatus::convert(
                    payment,
                    &timezone_config,
                    &now,
                )),
            })
            .collect(),
    }))
}
