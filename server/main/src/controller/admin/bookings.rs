use actix_web::web;
use dxe_data::queries::booking::{
    get_bookings_with_pending_cash_payment, get_bookings_with_pending_cash_refunds,
    get_confirmed_bookings,
};
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{Booking, BookingWithPayments, CashTransaction, Transaction};
use crate::models::handlers::admin::{GetBookingsQuery, GetBookingsResponse, GetBookingsType};
use crate::models::{Error, IntoView};
use crate::session::UserSession;
use crate::utils::datetime::is_in_effect;

pub async fn get(
    now: Now,
    _session: UserSession,
    query: web::Query<GetBookingsQuery>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetBookingsResponse>, Error> {
    let date_from = query.date_from.map(|v| v.to_utc());

    let mut connection = database.acquire().await?;

    let mut bookings: Vec<_> = match query.r#type {
        GetBookingsType::Pending => {
            get_bookings_with_pending_cash_payment(&mut connection, &now, false)
                .await?
                .into_iter()
                .map(|(booking, tx)| (booking, Some(tx)))
                .collect()
        }
        GetBookingsType::RefundPending => get_bookings_with_pending_cash_refunds(&mut connection)
            .await?
            .into_iter()
            .map(|(booking, tx)| (booking, Some(tx)))
            .collect(),
        GetBookingsType::Canceled => get_confirmed_bookings(&mut connection, &now, date_from)
            .await?
            .into_iter()
            .filter_map(|booking| {
                if is_in_effect(&booking.canceled_at, &now) {
                    Some((booking, None))
                } else {
                    None
                }
            })
            .collect(),
        GetBookingsType::Confirmed => get_confirmed_bookings(&mut connection, &now, date_from)
            .await?
            .into_iter()
            .filter_map(|booking| {
                if is_in_effect(&booking.confirmed_at, &now)
                    && !is_in_effect(&booking.canceled_at, &now)
                {
                    Some((booking, None))
                } else {
                    None
                }
            })
            .collect(),
    };

    match query.r#type {
        GetBookingsType::Confirmed => bookings.sort_by(|a, b| a.0.time_from.cmp(&b.0.time_from)),
        GetBookingsType::Canceled => bookings.sort_by(|a, b| {
            b.0.canceled_at
                .unwrap_or_default()
                .cmp(&a.0.canceled_at.unwrap_or_default())
        }),
        _ => {}
    }

    Ok(web::Json(GetBookingsResponse {
        bookings: bookings
            .into_iter()
            .map(|(booking, cash_tx)| -> Result<BookingWithPayments, Error> {
                Ok(BookingWithPayments {
                    booking: Booking::convert(booking, &timezone_config, &now)?
                        .finish(&booking_config, &now),
                    transaction: if let Some(cash_tx) = cash_tx {
                        Some(Transaction::Cash(CashTransaction::convert(
                            cash_tx,
                            &timezone_config,
                            &now,
                        )?))
                    } else {
                        None
                    },
                })
            })
            .collect::<Result<_, _>>()?,
    }))
}
