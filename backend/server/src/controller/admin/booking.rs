use actix_web::web;
use dxe_data::queries::booking::{
    cancel_booking, confirm_booking, get_audio_recording, get_booking, get_telemetry_files,
};
use dxe_data::queries::payment::{
    confirm_cash_payment, get_cash_transaction, get_toss_payments_transaction_by_product_id,
    refund_cash_payment,
};
use dxe_types::{BookingId, ProductId};
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{
    AudioRecording, Booking, BookingWithPayments, CashTransaction, TelemetryEntry,
    TossPaymentsTransaction, Transaction,
};
use crate::models::handlers::admin::{
    GetBookingResponse, ModifyAction, ModifyBookingRequest, ModifyBookingResponse,
};
use crate::models::{Error, IntoView};
use crate::services::calendar::CalendarService;
use crate::services::messaging::MessagingService;
use crate::utils::datetime::is_in_effect;

pub async fn get(
    now: Now,
    booking_id: web::Path<BookingId>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetBookingResponse>, Error> {
    let mut tx = database.begin().await?;

    let booking = get_booking(&mut tx, &booking_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    let product_id: ProductId = (*booking_id).into();

    let transaction = if let Some(transaction) =
        get_toss_payments_transaction_by_product_id(&mut tx, &product_id).await?
    {
        Some(Transaction::TossPayments(TossPaymentsTransaction::convert(
            transaction,
            &timezone_config,
            &now,
        )?))
    } else if let Some(transaction) = get_cash_transaction(&mut tx, &product_id).await? {
        Some(Transaction::Cash(CashTransaction::convert(
            transaction,
            &timezone_config,
            &now,
        )?))
    } else {
        None
    };

    let telemetry_files = get_telemetry_files(&mut tx, &booking_id).await?;
    let audio_recording = get_audio_recording(&mut tx, &booking_id).await?;

    Ok(web::Json(GetBookingResponse {
        booking: BookingWithPayments {
            booking: Booking::convert(booking, &timezone_config, &now)?
                .finish(&booking_config, &now),
            transaction,
        },
        telemetry_entries: telemetry_files
            .into_iter()
            .map(|v| TelemetryEntry::convert(v, &timezone_config, &now))
            .collect::<Result<_, _>>()?,
        audio_recording: audio_recording
            .map(|v| AudioRecording::convert(v, &timezone_config, &now))
            .transpose()?,
    }))
}

pub async fn put(
    now: Now,
    booking_id: web::Path<BookingId>,
    body: web::Json<ModifyBookingRequest>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
    messaging_service: web::Data<MessagingService>,
    calendar_service: web::Data<Option<CalendarService>>,
) -> Result<web::Json<ModifyBookingResponse>, Error> {
    let mut tx = database.begin().await?;

    let booking = get_booking(&mut tx, booking_id.as_ref())
        .await?
        .ok_or(Error::BookingNotFound)?;

    let product_id = ProductId::from(*booking_id);

    match body.action {
        ModifyAction::Confirm => {
            if !is_in_effect(&booking.canceled_at, &now)
                && confirm_booking(&mut tx, booking_id.as_ref(), &now).await?
            {
                confirm_cash_payment(&mut tx, &now, &product_id).await?;
            }
        }
        ModifyAction::Refund => {
            if is_in_effect(&booking.canceled_at, &now) {
                refund_cash_payment(&mut tx, &now, &product_id).await?;
            }
        }
        ModifyAction::Cancel => {
            if !is_in_effect(&booking.canceled_at, &now) {
                cancel_booking(&mut tx, &now, booking_id.as_ref()).await?;
                if let Some(calendar_service) = calendar_service.as_ref()
                    && let Err(e) = calendar_service.delete_booking(booking_id.as_ref()).await
                {
                    log::error!("Failed to delete event on calendar: {e}");
                }
            }
        }
    }

    let booking = get_booking(&mut tx, booking_id.as_ref())
        .await?
        .ok_or(Error::BookingNotFound)?;

    let cash_tx = get_cash_transaction(&mut tx, &product_id).await?;

    match body.action {
        ModifyAction::Confirm => {
            messaging_service
                .send_confirmation(&mut tx, booking.clone())
                .await?;
        }
        ModifyAction::Refund => {
            if let Some(cash_payment_status) = &cash_tx
                && let Some(refund_price) = cash_payment_status.refund_price
                && refund_price > 0
            {
                messaging_service.send_refund_confirmation(booking.clone(), refund_price);
            }
        }
        _ => {}
    }

    tx.commit().await?;

    Ok(web::Json(ModifyBookingResponse {
        booking: Booking::convert(booking, &timezone_config, &now)?.finish(&booking_config, &now),
        cash_transaction: if let Some(cash_tx) = cash_tx {
            Some(CashTransaction::convert(cash_tx, &timezone_config, &now)?)
        } else {
            None
        },
    }))
}
