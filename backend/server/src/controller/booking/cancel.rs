#![allow(clippy::too_many_arguments)]

use actix_web::web;
use dxe_data::queries::booking::{cancel_booking, get_booking_with_user_id};
use dxe_data::queries::payment::{
    get_cash_transaction, get_toss_payments_transaction_by_product_id,
    get_toss_payments_transactions_by_booking_amentments, refund_toss_payments,
    update_cash_refund_information,
};
use dxe_data::queries::user::update_user_cash_payment_refund_account;
use dxe_extern::toss_payments::{Error as TossPaymentsError, TossPaymentsClient};
use dxe_types::{BookingId, ProductId};
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{CashTransaction, TossPaymentsTransaction, Transaction};
use crate::models::handlers::booking::{CancelBookingRequest, CancelBookingResponse};
use crate::models::{Error, IntoView};
use crate::services::calendar::CalendarService;
use crate::services::messaging::MessagingService;
use crate::services::telemetry::{NotificationSender, Priority};
use crate::session::UserSession;

pub async fn delete(
    now: Now,
    session: UserSession,
    booking_id: web::Path<BookingId>,
    query: web::Query<CancelBookingRequest>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
    notification_sender: web::Data<NotificationSender>,
    messaging_service: web::Data<MessagingService>,
    calendar_service: web::Data<Option<CalendarService>>,
    toss_payments_service: web::Data<TossPaymentsClient>,
) -> Result<web::Json<CancelBookingResponse>, Error> {
    let mut tx = database.begin().await?;

    let booking = get_booking_with_user_id(&mut tx, booking_id.as_ref(), &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    cancel_booking(&mut tx, &now, booking_id.as_ref()).await?;

    let product_id = ProductId::from(*booking_id);

    let transaction = if let Some(mut cash_tx) = get_cash_transaction(&mut tx, &product_id).await? {
        let refund_price = booking_config
            .calculate_refund_price(
                timezone_config.as_ref(),
                cash_tx.price,
                booking.time_from,
                *now,
            )
            .map_err(|_| Error::NotRefundable)?;

        if refund_price > 0 && query.refund_account.is_none() {
            return Err(Error::RefundAccountRequired);
        }

        if update_cash_refund_information(
            &mut tx,
            &product_id,
            refund_price,
            query.refund_account.clone(),
        )
        .await?
        {
            cash_tx.refund_price = Some(refund_price);
            cash_tx.refund_account = query.refund_account.clone();
        }

        if let Some(refund_account) = &query.refund_account {
            let _ = update_user_cash_payment_refund_account(
                &mut tx,
                &session.user_id,
                Some(refund_account.as_str()),
            )
            .await?;
        }

        let refund_rate = (refund_price * 100 / cash_tx.price) as i32;

        messaging_service
            .send_cancellation(&mut tx, booking.clone(), refund_rate)
            .await?;

        Some(Transaction::Cash(CashTransaction::convert(
            cash_tx,
            &timezone_config,
            &now,
        )?))
    } else if let Some(mut toss_tx) =
        get_toss_payments_transaction_by_product_id(&mut tx, &product_id).await?
        && let Some(payment_key) = toss_tx.payment_key.as_ref()
    {
        let refund_price = booking_config
            .calculate_refund_price(
                timezone_config.as_ref(),
                toss_tx.price,
                booking.time_from,
                *now,
            )
            .map_err(|_| Error::NotRefundable)?;

        if refund_price > 0 {
            match toss_payments_service
                .cancel_payment(
                    payment_key,
                    query
                        .cancel_reason
                        .as_deref()
                        .unwrap_or("Cancellation request by user"),
                    Some(refund_price),
                )
                .await
            {
                Ok(_) => {
                    log::info!(
                        "Payment {payment_key} refunded successfully. Refunded amount: {refund_price}"
                    );
                }
                Err(e) => match e {
                    TossPaymentsError::Remote { code, message } => {
                        Err(Error::TossPaymentsFailed { message, code })?
                    }
                    TossPaymentsError::RemoteStatus(status) => {
                        Err(Error::PaymentFailed(status.to_string()))?
                    }
                    rest => Err(Error::Internal(Box::new(rest)))?,
                },
            }
        }

        if refund_toss_payments(&mut tx, &now, &toss_tx.id, refund_price).await? {
            toss_tx.refund_price = Some(refund_price);
            toss_tx.refunded_at = Some(*now);
        }

        for amendment in
            get_toss_payments_transactions_by_booking_amentments(&mut tx, &now, &booking_id).await?
        {
            let Some(payment_key) = amendment.payment_key else {
                continue;
            };

            let refund_price = booking_config
                .calculate_refund_price(
                    timezone_config.as_ref(),
                    amendment.price,
                    booking.time_from,
                    *now,
                )
                .map_err(|_| Error::NotRefundable)?;

            if refund_price > 0 {
                match toss_payments_service
                    .cancel_payment(
                        &payment_key,
                        query
                            .cancel_reason
                            .as_deref()
                            .unwrap_or("Cancellation request by user"),
                        Some(refund_price),
                    )
                    .await
                {
                    Ok(_) => {
                        log::info!(
                            "Amendment payment {payment_key} refunded successfully. Refunded amount: {refund_price}"
                        );
                    }
                    Err(e) => log::error!("Couldn't refund amendment tx {payment_key}: {e}"),
                }
            }
        }

        let refund_rate = (refund_price * 100 / toss_tx.price) as i32;
        messaging_service
            .send_cancellation(&mut tx, booking.clone(), refund_rate)
            .await?;

        Some(Transaction::TossPayments(TossPaymentsTransaction::convert(
            toss_tx,
            &timezone_config,
            &now,
        )?))
    } else {
        None
    };

    tx.commit().await?;

    if let Some(calendar_service) = calendar_service.as_ref()
        && let Err(e) = calendar_service.delete_booking(&booking.id).await
    {
        log::error!("Failed to delete event on calendar: {e}");
    }

    notification_sender.enqueue(
        Priority::High,
        format!(
            "Booking cancellation by {}: {}",
            booking.customer.name(),
            timezone_config.convert(booking.time_from),
        ),
    );

    Ok(web::Json(CancelBookingResponse { transaction }))
}
