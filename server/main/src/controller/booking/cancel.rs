#![allow(clippy::too_many_arguments)]

use actix_web::web;
use chrono::Utc;
use dxe_data::queries::booking::{
    cancel_booking, get_booking_with_user_id, get_cash_payment_status, update_refund_information,
};
use dxe_types::BookingId;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::models::entities::BookingCashPaymentStatus;
use crate::models::handlers::booking::{CancelBookingRequest, CancelBookingResponse};
use crate::models::{Error, IntoView};
use crate::services::messaging::biztalk::BiztalkSender;
use crate::services::telemetry::{NotificationSender, Priority};
use crate::session::UserSession;
use crate::utils::messaging::send_cancellation;

pub async fn delete(
    session: UserSession,
    booking_id: web::Path<BookingId>,
    query: web::Query<CancelBookingRequest>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
    notification_sender: web::Data<NotificationSender>,
    biztalk_sender: web::Data<Option<BiztalkSender>>,
) -> Result<web::Json<CancelBookingResponse>, Error> {
    let now = Utc::now();

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
                now,
            )
            .map_err(|_| Error::NotRefundable)?;

        if refund_price > 0 && query.refund_account.is_none() {
            return Err(Error::RefundAccountRequired);
        }

        if update_refund_information(
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

    tx.commit().await?;

    notification_sender.enqueue(
        Priority::High,
        format!(
            "Booking cancellation request by {}: {}",
            booking.customer.name(),
            timezone_config.convert(booking.time_from),
        ),
    );

    Ok(web::Json(CancelBookingResponse {
        cash_payment_status: cash_payment_status
            .map(|v| BookingCashPaymentStatus::convert(v, &timezone_config, &now)),
    }))
}
