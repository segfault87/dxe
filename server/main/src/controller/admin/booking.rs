use actix_web::web;
use chrono::Utc;
use dxe_data::queries::booking::{
    cancel_booking, confirm_booking, confirm_cash_payment, get_booking, get_cash_payment_status,
    refund_payment,
};
use dxe_types::BookingId;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::models::entities::{Booking, BookingCashPaymentStatus};
use crate::models::handlers::admin::{ModifyAction, ModifyBookingRequest, ModifyBookingResponse};
use crate::models::{Error, IntoView};
use crate::services::messaging::biztalk::BiztalkSender;
use crate::session::UserSession;
use crate::utils::datetime::is_in_effect;
use crate::utils::messaging::{send_confirmation, send_refund_confirmation};

pub async fn put(
    _session: UserSession,
    booking_id: web::Path<BookingId>,
    body: web::Json<ModifyBookingRequest>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
    biztalk_sender: web::Data<Option<BiztalkSender>>,
) -> Result<web::Json<ModifyBookingResponse>, Error> {
    let now = Utc::now();

    let mut tx = database.begin().await?;

    let booking = get_booking(&mut tx, booking_id.as_ref())
        .await?
        .ok_or(Error::BookingNotFound)?;

    if is_in_effect(&booking.confirmed_at, &now) {
        return Err(Error::BookingAlreadyConfirmed);
    }

    match body.action {
        ModifyAction::Confirm => {
            if !is_in_effect(&booking.canceled_at, &now)
                && confirm_booking(&mut tx, booking_id.as_ref(), &now).await?
            {
                confirm_cash_payment(&mut tx, &now, booking_id.as_ref()).await?;
            }
        }
        ModifyAction::Refund => {
            if is_in_effect(&booking.canceled_at, &now) {
                refund_payment(&mut tx, &now, booking_id.as_ref()).await?;
            }
        }
        ModifyAction::Cancel => {
            if !is_in_effect(&booking.canceled_at, &now) {
                cancel_booking(&mut tx, &now, booking_id.as_ref()).await?;
            }
        }
    }

    let booking = get_booking(&mut tx, booking_id.as_ref())
        .await?
        .ok_or(Error::BookingNotFound)?;

    let cash_payment_status = get_cash_payment_status(&mut tx, booking_id.as_ref()).await?;

    match body.action {
        ModifyAction::Confirm => {
            send_confirmation(biztalk_sender.as_ref(), &mut tx, &timezone_config, &booking).await?;
        }
        ModifyAction::Refund => {
            if let Some(cash_payment_status) = &cash_payment_status
                && let Some(refund_price) = cash_payment_status.refund_price
                && refund_price > 0
            {
                send_refund_confirmation(
                    biztalk_sender.as_ref(),
                    &timezone_config,
                    &booking,
                    refund_price,
                );
            }
        }
        _ => {}
    }

    tx.commit().await?;

    Ok(web::Json(ModifyBookingResponse {
        booking: Booking::convert(booking, &timezone_config, &now)?.finish(&booking_config, &now),
        cash_payment_status: if let Some(cash_payment_status) = cash_payment_status {
            Some(BookingCashPaymentStatus::convert(
                cash_payment_status,
                &timezone_config,
                &now,
            )?)
        } else {
            None
        },
    }))
}
