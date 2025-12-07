#![allow(clippy::too_many_arguments)]

use actix_web::web;
use dxe_data::queries::booking::{
    cancel_booking, get_booking_with_user_id, get_cash_payment_status,
    get_toss_payment_status_by_booking_id, refund_toss_payment, update_cash_refund_information,
};
use dxe_data::queries::user::update_user_cash_payment_refund_account;
use dxe_extern::toss_payments::{Error as TossPaymentsError, TossPaymentsClient};
use dxe_types::BookingId;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{BookingCashPaymentStatus, BookingTossPaymentStatus};
use crate::models::handlers::booking::{CancelBookingRequest, CancelBookingResponse};
use crate::models::{Error, IntoView};
use crate::services::calendar::CalendarService;
use crate::services::messaging::biztalk::BiztalkSender;
use crate::services::telemetry::{NotificationSender, Priority};
use crate::session::UserSession;
use crate::utils::messaging::send_cancellation;

pub async fn delete(
    now: Now,
    session: UserSession,
    booking_id: web::Path<BookingId>,
    query: web::Query<CancelBookingRequest>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
    notification_sender: web::Data<NotificationSender>,
    biztalk_sender: web::Data<Option<BiztalkSender>>,
    calendar_service: web::Data<Option<CalendarService>>,
    toss_payments_service: web::Data<TossPaymentsClient>,
) -> Result<web::Json<CancelBookingResponse>, Error> {
    let mut tx = database.begin().await?;

    let booking = get_booking_with_user_id(&mut tx, booking_id.as_ref(), &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    cancel_booking(&mut tx, &now, booking_id.as_ref()).await?;

    let mut cash_payment_status = get_cash_payment_status(&mut tx, booking_id.as_ref()).await?;
    if let Some(cash_payment_status) = &mut cash_payment_status {
        let refund_price = booking_config
            .calculate_refund_price(
                timezone_config.as_ref(),
                cash_payment_status.price,
                booking.time_from,
                *now,
            )
            .map_err(|_| Error::NotRefundable)?;

        if refund_price > 0 && query.refund_account.is_none() {
            return Err(Error::RefundAccountRequired);
        }

        if update_cash_refund_information(
            &mut tx,
            booking_id.as_ref(),
            refund_price,
            query.refund_account.clone(),
        )
        .await?
        {
            cash_payment_status.refund_price = Some(refund_price);
            cash_payment_status.refund_account = query.refund_account.clone();
        }

        if let Some(refund_account) = &query.refund_account {
            let _ = update_user_cash_payment_refund_account(
                &mut tx,
                &session.user_id,
                Some(refund_account.as_str()),
            )
            .await?;
        }

        let refund_rate = (refund_price * 100 / cash_payment_status.price) as i32;

        send_cancellation(
            biztalk_sender.as_ref(),
            &mut tx,
            &timezone_config,
            &booking,
            refund_rate,
        )
        .await?;
    }

    let mut toss_payment_status =
        get_toss_payment_status_by_booking_id(&mut tx, &booking_id).await?;
    if let Some(toss_payment_status) = &mut toss_payment_status
        && let Some(payment_key) = toss_payment_status.payment_key.as_ref()
    {
        let refund_price = booking_config
            .calculate_refund_price(
                timezone_config.as_ref(),
                toss_payment_status.price,
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

        if refund_toss_payment(&mut tx, &now, &toss_payment_status.id, refund_price).await? {
            toss_payment_status.refund_price = Some(refund_price);
            toss_payment_status.refunded_at = Some(*now);
        }

        let refund_rate = (refund_price * 100 / toss_payment_status.price) as i32;
        send_cancellation(
            biztalk_sender.as_ref(),
            &mut tx,
            &timezone_config,
            &booking,
            refund_rate,
        )
        .await?;
    }

    tx.commit().await?;

    if let Some(calendar_service) = calendar_service.as_ref() {
        if let Err(e) = calendar_service.delete_booking(&booking.id).await {
            log::error!("Failed to delete event on calendar: {e}");
        }
    }

    notification_sender.enqueue(
        Priority::High,
        format!(
            "Booking cancellation by {}: {}",
            booking.customer.name(),
            timezone_config.convert(booking.time_from),
        ),
    );

    Ok(web::Json(CancelBookingResponse {
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
