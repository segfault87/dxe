use actix_web::web;
use chrono::{DateTime, TimeDelta};
use dxe_data::queries::booking::{
    get_bookings_with_pending_cash_payment, get_bookings_with_pending_cash_refunds,
    get_complete_bookings, get_confirmed_bookings,
};
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{Booking, BookingWithPayments, CashTransaction, Transaction};
use crate::models::handlers::admin::{GetBookingsQuery, GetBookingsResponse, GetBookingsType};
use crate::models::{Error, IntoView};
use crate::utils::datetime::is_in_effect;

pub async fn get(
    now: Now,
    query: web::Query<GetBookingsQuery>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetBookingsResponse>, Error> {
    let date_from = query
        .date_from
        .map(|v| v.to_utc())
        .unwrap_or(DateTime::UNIX_EPOCH);
    let date_to = query
        .date_to
        .map(|v| v.to_utc())
        .unwrap_or(now.to_utc() + TimeDelta::days(365));
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(30);

    let mut connection = database.acquire().await?;

    let mut bookings: Vec<_> = match query.r#type {
        GetBookingsType::Complete => {
            get_complete_bookings(&mut connection, &now, &date_from, &date_to, offset, limit)
                .await?
                .into_iter()
                .map(|v| (v, None))
                .collect()
        }
        GetBookingsType::Pending => get_bookings_with_pending_cash_payment(
            &mut connection,
            &now,
            false,
            &date_from,
            &date_to,
            offset,
            limit,
        )
        .await?
        .into_iter()
        .map(|(booking, tx)| (booking, Some(tx)))
        .collect(),
        GetBookingsType::RefundPending => get_bookings_with_pending_cash_refunds(
            &mut connection,
            &date_from,
            &date_to,
            offset,
            limit,
        )
        .await?
        .into_iter()
        .map(|(booking, tx)| (booking, Some(tx)))
        .collect(),
        GetBookingsType::Canceled => {
            get_confirmed_bookings(&mut connection, &now, &date_from, &date_to, offset, limit)
                .await?
                .into_iter()
                .filter_map(|booking| {
                    if is_in_effect(&booking.canceled_at, &now) {
                        Some((booking, None))
                    } else {
                        None
                    }
                })
                .collect()
        }
        GetBookingsType::Confirmed => {
            get_confirmed_bookings(&mut connection, &now, &date_from, &date_to, offset, limit)
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
                .collect()
        }
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
