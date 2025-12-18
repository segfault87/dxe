use actix_web::web;
use chrono::TimeDelta;
use dxe_data::queries::booking::{get_booking_with_user_id, get_occupied_slots};
use dxe_data::queries::payment::{
    get_cash_transaction, get_toss_payments_transaction_by_product_id,
    get_toss_payments_transactions_by_booking_amentments,
};
use dxe_types::{BookingId, ProductId};
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{Booking, CashTransaction, TossPaymentsTransaction, Transaction};
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

    let product_id = ProductId::from(booking.id);

    let transaction = if let Some(mut toss_tx) =
        get_toss_payments_transaction_by_product_id(&mut tx, &product_id).await?
    {
        let amendment_txs =
            get_toss_payments_transactions_by_booking_amentments(&mut tx, &now, &booking_id)
                .await?;

        for tx in amendment_txs {
            toss_tx.price += tx.price;
        }

        Some(Transaction::TossPayments(TossPaymentsTransaction::convert(
            toss_tx,
            &timezone_config,
            &now,
        )?))
    } else if let Some(cash_tx) = get_cash_transaction(&mut tx, &product_id).await? {
        Some(Transaction::Cash(CashTransaction::convert(
            cash_tx,
            &timezone_config,
            &now,
        )?))
    } else {
        None
    };

    let amendable = booking_config
        .calculate_refund_price(&timezone_config, 100, booking.time_from, *now)
        .map(|v| v != 0)
        .unwrap_or(false);

    let hours = (booking.time_to - booking.time_from).num_hours();
    let mut extendable_hours = booking_config.max_booking_hours - hours;

    if extendable_hours > 0 {
        let start = booking.time_to;
        let end = start + TimeDelta::hours(extendable_hours);
        let mut slots = get_occupied_slots(
            &mut tx,
            &now,
            &booking.unit_id,
            &start,
            &end,
            Some(booking_id.as_ref()),
            None,
        )
        .await?;
        slots.sort_by(|a, b| a.time_from.cmp(&b.time_from));

        if let Some(first) = slots.first() {
            extendable_hours = std::cmp::min(
                extendable_hours,
                (first.time_from - booking.time_to).num_hours(),
            );
        }
    }

    Ok(web::Json(GetBookingResponse {
        booking: Booking::convert(booking, &timezone_config, &now)?
            .finish(booking_config.as_ref(), &now),
        transaction,
        amendable,
        extendable_hours,
    }))
}
