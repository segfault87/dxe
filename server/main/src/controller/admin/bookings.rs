use actix_web::web;
use chrono::Utc;
use dxe_data::queries::booking::{
    get_bookings_pending, get_bookings_refund_pending, get_confirmed_bookings_with_payments,
};
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::models::entities::{Booking, BookingCashPaymentStatus, BookingWithPayments};
use crate::models::handlers::admin::{GetBookingsQuery, GetBookingsResponse, GetBookingsType};
use crate::models::{Error, IntoView};
use crate::session::UserSession;
use crate::utils::datetime::is_in_effect;

pub async fn get(
    _session: UserSession,
    query: web::Query<GetBookingsQuery>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetBookingsResponse>, Error> {
    let now = Utc::now();

    let date_from = query.date_from.map(|v| v.to_utc());

    let mut connection = database.acquire().await?;

    let bookings = match query.r#type {
        GetBookingsType::Pending => get_bookings_pending(&mut connection, &now, false).await?,
        GetBookingsType::RefundPending => get_bookings_refund_pending(&mut connection)
            .await?
            .into_iter()
            .map(|(b, payment)| (b, Some(payment)))
            .collect(),
        GetBookingsType::Canceled => {
            get_confirmed_bookings_with_payments(&mut connection, &now, date_from)
                .await?
                .into_iter()
                .filter(|(booking, _)| is_in_effect(&booking.canceled_at, &now))
                .collect()
        }
        GetBookingsType::Confirmed => {
            get_confirmed_bookings_with_payments(&mut connection, &now, date_from)
                .await?
                .into_iter()
                .filter(|(booking, _)| {
                    is_in_effect(&booking.confirmed_at, &now)
                        && !is_in_effect(&booking.canceled_at, &now)
                })
                .collect()
        }
    };

    Ok(web::Json(GetBookingsResponse {
        bookings: bookings
            .into_iter()
            .map(|(booking, payment)| BookingWithPayments {
                booking: Booking::convert(booking, &timezone_config, &now)
                    .finish(&booking_config, &now),
                payment: payment
                    .map(|v| BookingCashPaymentStatus::convert(v, &timezone_config, &now)),
            })
            .collect(),
    }))
}
