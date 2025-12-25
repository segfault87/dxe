use actix_web::web;
use chrono::TimeDelta;
use dxe_data::entities::{
    AdhocReservation, BookingAmendment, Identity, Product, TossPaymentsTransaction,
};
use dxe_data::queries::booking::{
    cancel_booking_amendment, confirm_booking_amendment, create_adhoc_reservation, create_booking,
    expire_adhoc_reservation, get_adhoc_reservation, get_booking, get_booking_with_user_id,
    get_product, update_booking_time,
};
use dxe_data::queries::identity::{get_group_members, get_identity, is_member_of};
use dxe_data::queries::payment::{
    confirm_toss_payments_transaction, create_toss_payments_transaction,
    get_toss_payments_transaction_by_id, get_toss_payments_transaction_by_temporary_reservation_id,
};
use dxe_data::queries::unit::is_unit_enabled;
use dxe_extern::toss_payments::{Error as TossPaymentsError, TossPaymentsClient};
use dxe_types::{BookingId, ForeignPaymentId, ProductId};
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::Error;
use crate::models::entities::ProductType;
use crate::models::handlers::booking::{
    ConfirmTossPaymentRequest, ConfirmTossPaymentResponse, GetTossPaymentStateResponse,
    InitiateTossPaymentRequest, InitiateTossPaymentResponse,
};
use crate::services::calendar::CalendarService;
use crate::services::messaging::MessagingService;
use crate::services::notification::{NotificationSender, Priority};
use crate::session::UserSession;
use crate::utils::datetime::{is_in_effect, truncate_time};

const TEMPORARY_RESERVATION_LIFE: TimeDelta = TimeDelta::minutes(5);

pub async fn post(
    now: Now,
    session: UserSession,
    body: web::Json<InitiateTossPaymentRequest>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<InitiateTossPaymentResponse>, Error> {
    let mut tx = database.begin().await?;

    if is_unit_enabled(&mut tx, &body.unit_id).await? != Some(true) {
        return Err(Error::UnitNotFound);
    }

    if body.desired_hours > booking_config.max_booking_hours {
        return Err(Error::InvalidTimeRange);
    }

    let time_from = truncate_time(body.time_from).to_utc();
    let time_to = time_from + TimeDelta::hours(body.desired_hours);

    let price = booking_config
        .calculate_price(&body.unit_id, time_from, time_to)
        .map_err(|_| Error::UnitNotFound)?;

    let identity = get_identity(&mut tx, &now, &body.identity_id)
        .await?
        .ok_or(Error::UserNotFound)?;

    let expires_in = *now + TEMPORARY_RESERVATION_LIFE;

    let (id, temporary_reservation_id) = if let Some(temporary_reservation_id) =
        body.temporary_reservation_id
        && let Some(temporary_reservation) =
            get_adhoc_reservation(&mut tx, &temporary_reservation_id).await?
    {
        if temporary_reservation.holder.id != session.user_id {
            return Err(Error::TimeRangeOccupied);
        }

        if temporary_reservation.time_from != time_from || temporary_reservation.time_to != time_to
        {
            // Update temporary reservation (when date/time changes)

            expire_adhoc_reservation(&mut tx, &now, &temporary_reservation.id).await?;

            let new_temporary_reservation_id = create_adhoc_reservation(
                &mut tx,
                &now,
                &body.unit_id,
                &identity.id(),
                &session.user_id,
                &time_from,
                &time_to,
                &Some(String::from("Temporary preemption during Toss payment")),
                &Some(expires_in),
            )
            .await?;

            let id = ForeignPaymentId::generate();
            let _ = create_toss_payments_transaction(
                &mut tx,
                &now,
                &id,
                &session.user_id,
                Some(&temporary_reservation_id),
                None,
                price,
            )
            .await?;

            (id, new_temporary_reservation_id)
        } else {
            // Reuse existing temporary reservation

            let toss_tx = get_toss_payments_transaction_by_temporary_reservation_id(
                &mut tx,
                &temporary_reservation_id,
            )
            .await?;

            if let Some(toss_tx) = toss_tx {
                (toss_tx.id, temporary_reservation_id)
            } else {
                let id = ForeignPaymentId::generate();
                let _ = create_toss_payments_transaction(
                    &mut tx,
                    &now,
                    &id,
                    &session.user_id,
                    Some(&temporary_reservation_id),
                    None,
                    price,
                )
                .await?;

                (id, temporary_reservation_id)
            }
        }
    } else {
        // Create new reservation

        let temporary_reservation_id = create_adhoc_reservation(
            &mut tx,
            &now,
            &body.unit_id,
            &identity.id(),
            &session.user_id,
            &time_from,
            &time_to,
            &Some(String::from("Temporary preemption during Toss payment")),
            &Some(expires_in),
        )
        .await?;

        let id = ForeignPaymentId::generate();

        let _ = create_toss_payments_transaction(
            &mut tx,
            &now,
            &id,
            &session.user_id,
            Some(&temporary_reservation_id),
            None,
            price,
        )
        .await?;

        (id, temporary_reservation_id)
    };

    tx.commit().await?;

    Ok(web::Json(InitiateTossPaymentResponse {
        order_id: id,
        price,
        temporary_reservation_id,
        expires_in: timezone_config.convert(expires_in),
    }))
}

async fn confirm_booking_payment<'tx>(
    now: &Now,
    body: &ConfirmTossPaymentRequest,
    session: &UserSession,
    toss_tx: TossPaymentsTransaction,
    temporary_reservation: AdhocReservation,
    tx: &mut sqlx::SqliteTransaction<'tx>,
    toss_payments_client: &TossPaymentsClient,
    calendar_service: &Option<CalendarService>,
    timezone_config: &TimeZoneConfig,
    notification_sender: &NotificationSender,
    messaging_service: &MessagingService,
) -> Result<BookingId, Error> {
    if temporary_reservation.holder.id != session.user_id {
        return Err(Error::Forbidden);
    }

    if is_in_effect(&toss_tx.confirmed_at, now) || toss_tx.product_id.is_some() {
        return Err(Error::BookingAlreadyConfirmed)?;
    }

    if body.amount != toss_tx.price {
        return Err(Error::PaymentFailed(String::from(
            "거래승인된 액수와 결재금액이 다릅니다.",
        )));
    }

    let payment = match toss_payments_client
        .confirm_payment(&body.order_id, body.amount, &body.payment_key)
        .await
    {
        Ok(v) => {
            log::info!(
                "Payment {} processed successfully. total amount: {}",
                v.order_id,
                v.total_amount,
            );

            v
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
    };

    expire_adhoc_reservation(tx, now, &temporary_reservation.id).await?;
    if let Some(calendar_service) = calendar_service.as_ref() {
        let _ = calendar_service
            .delete_adhoc_reservation(&temporary_reservation.id)
            .await;
    }

    let booking_id = create_booking(
        tx,
        now,
        &temporary_reservation.unit_id,
        &session.user_id,
        &temporary_reservation.customer.id(),
        &temporary_reservation.time_from,
        &temporary_reservation.time_to,
        true,
    )
    .await?;

    let customers = match &temporary_reservation.customer {
        Identity::User(u) => {
            if u.id != session.user_id {
                return Err(Error::UserNotFound);
            }
            vec![u.clone()]
        }
        Identity::Group(g) => {
            if !is_member_of(tx, &g.id, &session.user_id).await? {
                return Err(Error::GroupNotFound);
            }
            get_group_members(tx, &g.id).await?
        }
    };

    let booking = get_booking_with_user_id(tx, &booking_id, &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    if let Some(calendar_service) = calendar_service.as_ref()
        && let Err(e) = calendar_service
            .register_booking(&booking, &customers)
            .await
    {
        log::error!("Failed to register event on calendar: {e}");
    }

    let product_id = ProductId::from(booking.id);

    confirm_toss_payments_transaction(tx, now, &body.order_id, &product_id, &payment.payment_key)
        .await?;

    let desired_hours =
        (temporary_reservation.time_to - temporary_reservation.time_from).num_hours();
    notification_sender.enqueue(
        Priority::High,
        format!(
            "New booking by {}: {} ({} hours)",
            temporary_reservation.customer.name(),
            timezone_config.convert(temporary_reservation.time_from),
            desired_hours
        ),
    );

    messaging_service
        .send_confirmation(tx, booking.clone())
        .await?;

    Ok(booking_id)
}

async fn confirm_amend_payment<'tx>(
    now: &Now,
    body: &ConfirmTossPaymentRequest,
    toss_tx: TossPaymentsTransaction,
    booking_amendment: BookingAmendment,
    tx: &mut sqlx::SqliteTransaction<'tx>,
    toss_payments_client: &TossPaymentsClient,
    calendar_service: &Option<CalendarService>,
    timezone_config: &TimeZoneConfig,
    notification_sender: &NotificationSender,
    messaging_service: &MessagingService,
) -> Result<BookingId, Error> {
    let booking = get_booking(tx, &booking_amendment.booking_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    if is_in_effect(&toss_tx.confirmed_at, now)
        || is_in_effect(&booking_amendment.confirmed_at, now)
    {
        return Err(Error::BookingAlreadyConfirmed)?;
    }

    if body.amount != toss_tx.price {
        return Err(Error::PaymentFailed(String::from(
            "거래승인된 액수와 결재금액이 다릅니다.",
        )));
    }

    let _ = update_booking_time(
        tx,
        now,
        &booking.id,
        &booking_amendment.desired_time_from,
        &booking_amendment.desired_time_to,
    )
    .await?;

    let _ = confirm_booking_amendment(tx, now, &booking_amendment.id).await?;

    let payment = match toss_payments_client
        .confirm_payment(&body.order_id, body.amount, &body.payment_key)
        .await
    {
        Ok(v) => {
            log::info!(
                "Payment {} processed successfully. total amount: {}",
                v.order_id,
                v.total_amount,
            );

            v
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
    };

    let product_id = ProductId::from(booking_amendment.id);
    let _ =
        confirm_toss_payments_transaction(tx, now, &toss_tx.id, &product_id, &payment.payment_key)
            .await?;

    if let Err(e) = messaging_service
        .send_amend_notification(
            tx,
            booking.clone(),
            booking_amendment.desired_time_from,
            booking_amendment.desired_time_to,
        )
        .await
    {
        log::warn!("Could not send amend notification to customers: {e}");
    }

    if let Some(calendar_service) = calendar_service.as_ref() {
        let mut updated_booking = booking.clone();
        updated_booking.time_from = booking_amendment.desired_time_from;
        updated_booking.time_to = booking_amendment.desired_time_to;

        if let Err(e) = calendar_service.update_booking_time(&booking).await {
            log::warn!("Could not update booking {} to calendar: {e}", booking.id);
        }
    }

    notification_sender.enqueue(
        Priority::High,
        format!(
            "Booking by {} moved from {} - {} to {} - {}",
            booking.customer.name(),
            timezone_config.convert(booking.time_from),
            timezone_config.convert(booking.time_to),
            timezone_config.convert(booking_amendment.desired_time_from),
            timezone_config.convert(booking_amendment.desired_time_to),
        ),
    );

    Ok(booking.id)
}

pub async fn confirm_payment(
    now: Now,
    session: UserSession,
    body: web::Json<ConfirmTossPaymentRequest>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
    toss_payments_client: web::Data<TossPaymentsClient>,
    calendar_service: web::Data<Option<CalendarService>>,
    messaging_service: web::Data<MessagingService>,
    notification_sender: web::Data<NotificationSender>,
) -> Result<web::Json<ConfirmTossPaymentResponse>, Error> {
    let mut tx = database.begin().await?;

    let toss_tx = get_toss_payments_transaction_by_id(&mut tx, &body.order_id)
        .await?
        .ok_or(Error::ForeignPaymentNotFound)?;

    if toss_tx.user_id != session.user_id {
        return Err(Error::Forbidden);
    }

    if is_in_effect(&toss_tx.confirmed_at, &now) {
        return Err(Error::BookingAlreadyConfirmed);
    }

    let booking_id = if let Some(temporary_reservation_id) = toss_tx.temporary_reservation_id {
        let temporary_reservation = get_adhoc_reservation(&mut tx, &temporary_reservation_id)
            .await?
            .ok_or(Error::PaymentFailed(String::from(
                "임시 예약을 찾을 수 없습니다.",
            )))?;

        confirm_booking_payment(
            &now,
            &body,
            &session,
            toss_tx,
            temporary_reservation,
            &mut tx,
            toss_payments_client.as_ref(),
            calendar_service.as_ref(),
            timezone_config.as_ref(),
            notification_sender.as_ref(),
            messaging_service.as_ref(),
        )
        .await?
    } else if let Some(product_id) = toss_tx.product_id {
        let Some(Product::Amendment(booking_amendment)) = get_product(&mut tx, &product_id).await?
        else {
            return Err(Error::PaymentFailed(String::from(
                "예약 변경 정보를 찾을 수 없습니다.",
            )));
        };

        confirm_amend_payment(
            &now,
            &body,
            toss_tx,
            booking_amendment,
            &mut tx,
            toss_payments_client.as_ref(),
            calendar_service.as_ref(),
            timezone_config.as_ref(),
            notification_sender.as_ref(),
            messaging_service.as_ref(),
        )
        .await?
    } else {
        return Err(Error::PaymentFailed(String::from(
            "잘못된 주문 정보입니다.",
        )));
    };

    tx.commit().await?;

    Ok(web::Json(ConfirmTossPaymentResponse { booking_id }))
}

pub async fn get(
    session: UserSession,
    foreign_payment_id: web::Path<ForeignPaymentId>,
    timezone_config: web::Data<TimeZoneConfig>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<GetTossPaymentStateResponse>, Error> {
    let mut tx = database.begin().await?;

    let toss_tx = get_toss_payments_transaction_by_id(&mut tx, &foreign_payment_id)
        .await?
        .ok_or(Error::ForeignPaymentNotFound)?;

    if let Some(temporary_reservation_id) = toss_tx.temporary_reservation_id {
        let adhoc_reservation = get_adhoc_reservation(&mut tx, &temporary_reservation_id)
            .await?
            .ok_or(Error::ForeignPaymentNotFound)?;

        if toss_tx.user_id != session.user_id || adhoc_reservation.holder.id != session.user_id {
            return Err(Error::ForeignPaymentNotFound);
        }

        Ok(web::Json(GetTossPaymentStateResponse {
            r#type: ProductType::Booking,
            time_from: timezone_config.convert(adhoc_reservation.time_from),
            time_to: timezone_config.convert(adhoc_reservation.time_to),
        }))
    } else if let Some(product_id) = toss_tx.product_id {
        let Some(Product::Amendment(amendment)) = get_product(&mut tx, &product_id).await? else {
            return Err(Error::BookingAmendmentNotFound);
        };

        Ok(web::Json(GetTossPaymentStateResponse {
            r#type: ProductType::BookingAmendment,
            time_from: timezone_config.convert(amendment.desired_time_from),
            time_to: timezone_config.convert(amendment.desired_time_to),
        }))
    } else {
        Err(Error::ForeignPaymentNotFound)
    }
}

pub async fn delete(
    now: Now,
    session: UserSession,
    foreign_payment_id: web::Path<ForeignPaymentId>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let toss_tx = get_toss_payments_transaction_by_id(&mut tx, &foreign_payment_id)
        .await?
        .ok_or(Error::ForeignPaymentNotFound)?;

    if let Some(temporary_reservation_id) = toss_tx.temporary_reservation_id {
        let temporary_reservation = get_adhoc_reservation(&mut tx, &temporary_reservation_id)
            .await?
            .ok_or(Error::ForeignPaymentNotFound)?;

        if toss_tx.user_id != session.user_id || temporary_reservation.holder.id != session.user_id
        {
            return Err(Error::Forbidden);
        }

        if !is_in_effect(&temporary_reservation.deleted_at, &now) {
            let _ = expire_adhoc_reservation(&mut tx, &now, &temporary_reservation.id).await?;
        }
    } else if let Some(product_id) = toss_tx.product_id {
        let Some(Product::Amendment(amendment)) = get_product(&mut tx, &product_id).await? else {
            return Err(Error::BookingAmendmentNotFound);
        };

        if !is_in_effect(&amendment.canceled_at, &now) {
            let _ = cancel_booking_amendment(&mut tx, &now, &amendment.id).await?;
        }
    }

    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}
