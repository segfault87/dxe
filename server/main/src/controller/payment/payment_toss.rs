use actix_web::web;
use chrono::TimeDelta;
use dxe_data::entities::Identity;
use dxe_data::queries::booking::{
    confirm_toss_payment_status, create_adhoc_reservation, create_booking,
    create_toss_payment_status, expire_adhoc_reservation, get_adhoc_reservation,
    get_booking_with_user_id, get_toss_payment_status_by_id,
    get_toss_payment_status_by_temporary_reservation_id, is_booking_available,
};
use dxe_data::queries::identity::{get_group_members, get_identity, is_member_of};
use dxe_data::queries::unit::is_unit_enabled;
use dxe_extern::toss_payments::{Error as TossPaymentsError, TossPaymentsClient};
use dxe_types::ForeignPaymentId;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::Error;
use crate::models::handlers::booking::{
    ConfirmTossPaymentRequest, ConfirmTossPaymentResponse, GetTossPaymentStateResponse,
    InitiateTossPaymentRequest, InitiateTossPaymentResponse,
};
use crate::services::calendar::CalendarService;
use crate::services::messaging::biztalk::BiztalkSender;
use crate::session::UserSession;
use crate::utils::datetime::{is_in_effect, truncate_time};
use crate::utils::messaging::send_confirmation;

const TEMPORARY_RESERVATION_LIFE: TimeDelta = TimeDelta::minutes(10);

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
            let _ = create_toss_payment_status(
                &mut tx,
                &now,
                &id,
                &session.user_id,
                &temporary_reservation_id,
                &None,
                price,
            )
            .await?;

            (id, new_temporary_reservation_id)
        } else {
            // Reuse existing temporary reservation

            let toss_payment_status = get_toss_payment_status_by_temporary_reservation_id(
                &mut tx,
                &temporary_reservation_id,
            )
            .await?;

            if let Some(toss_payment_status) = toss_payment_status {
                (toss_payment_status.id, temporary_reservation_id)
            } else {
                let id = ForeignPaymentId::generate();
                let _ = create_toss_payment_status(
                    &mut tx,
                    &now,
                    &id,
                    &session.user_id,
                    &temporary_reservation_id,
                    &None,
                    price,
                )
                .await?;

                (id, temporary_reservation_id)
            }
        }
    } else {
        // Create new reservation

        if !is_booking_available(&mut tx, &now, &body.unit_id, &time_from, &time_to).await? {
            return Err(Error::TimeRangeOccupied);
        }

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

        let _ = create_toss_payment_status(
            &mut tx,
            &now,
            &id,
            &session.user_id,
            &temporary_reservation_id,
            &None,
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

pub async fn confirm_payment(
    now: Now,
    session: UserSession,
    body: web::Json<ConfirmTossPaymentRequest>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
    toss_payments_client: web::Data<TossPaymentsClient>,
    calendar_service: web::Data<Option<CalendarService>>,
    biztalk_sender: web::Data<Option<BiztalkSender>>,
) -> Result<web::Json<ConfirmTossPaymentResponse>, Error> {
    let mut tx = database.begin().await?;

    let toss_payment_status = get_toss_payment_status_by_id(&mut tx, &body.order_id)
        .await?
        .ok_or(Error::ForeignPaymentNotFound)?;

    let temporary_reservation =
        get_adhoc_reservation(&mut tx, &toss_payment_status.temporary_reservation_id)
            .await?
            .ok_or(Error::PaymentFailed(String::from(
                "임시 예약을 찾을 수 없습니다.",
            )))?;

    if toss_payment_status.user_id != session.user_id
        || temporary_reservation.holder.id != session.user_id
    {
        return Err(Error::Forbidden);
    }

    if is_in_effect(&toss_payment_status.confirmed_at, &now)
        || toss_payment_status.booking_id.is_some()
    {
        return Err(Error::BookingAlreadyConfirmed)?;
    }

    if body.amount != toss_payment_status.price {
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

    expire_adhoc_reservation(&mut tx, &now, &toss_payment_status.temporary_reservation_id).await?;
    if let Some(calendar_service) = calendar_service.as_ref() {
        let _ = calendar_service
            .delete_adhoc_reservation(&toss_payment_status.temporary_reservation_id)
            .await;
    }

    let booking_id = create_booking(
        &mut tx,
        &now,
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
            if !is_member_of(&mut tx, &g.id, &session.user_id).await? {
                return Err(Error::GroupNotFound);
            }
            get_group_members(&mut tx, &g.id).await?
        }
    };

    let booking = get_booking_with_user_id(&mut tx, &booking_id, &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    if let Some(calendar_service) = calendar_service.as_ref()
        && let Err(e) = calendar_service
            .register_booking(&timezone_config, &booking, &customers)
            .await
    {
        log::error!("Failed to register event on calendar: {e}");
    }

    confirm_toss_payment_status(
        &mut tx,
        &now,
        &body.order_id,
        &booking_id,
        &payment.payment_key,
    )
    .await?;

    send_confirmation(biztalk_sender.as_ref(), &mut tx, &timezone_config, &booking).await?;

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

    let toss_payment_status = get_toss_payment_status_by_id(&mut tx, &foreign_payment_id)
        .await?
        .ok_or(Error::ForeignPaymentNotFound)?;

    let adhoc_reservation =
        get_adhoc_reservation(&mut tx, &toss_payment_status.temporary_reservation_id)
            .await?
            .ok_or(Error::ForeignPaymentNotFound)?;

    if toss_payment_status.user_id != session.user_id
        || adhoc_reservation.holder.id != session.user_id
    {
        return Err(Error::ForeignPaymentNotFound);
    }

    Ok(web::Json(GetTossPaymentStateResponse {
        time_from: timezone_config.convert(adhoc_reservation.time_from),
        time_to: timezone_config.convert(adhoc_reservation.time_to),
    }))
}

pub async fn delete(
    now: Now,
    session: UserSession,
    foreign_payment_id: web::Path<ForeignPaymentId>,
    database: web::Data<SqlitePool>,
    calendar_service: web::Data<Option<CalendarService>>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let toss_payment_status = get_toss_payment_status_by_id(&mut tx, &foreign_payment_id)
        .await?
        .ok_or(Error::ForeignPaymentNotFound)?;

    let temporary_reservation =
        get_adhoc_reservation(&mut tx, &toss_payment_status.temporary_reservation_id)
            .await?
            .ok_or(Error::ForeignPaymentNotFound)?;

    if toss_payment_status.user_id != session.user_id
        || temporary_reservation.holder.id != session.user_id
    {
        return Err(Error::Forbidden);
    }

    if !is_in_effect(&temporary_reservation.deleted_at, &now)
        && expire_adhoc_reservation(&mut tx, &now, &temporary_reservation.id).await?
    {
        if let Some(calendar_service) = calendar_service.as_ref() {
            if let Err(e) = calendar_service
                .delete_adhoc_reservation(&temporary_reservation.id)
                .await
            {
                log::error!(
                    "Error while deleting temporary reservation: {}: {e}",
                    temporary_reservation.id
                );
            }
        }
    }

    Ok(web::Json(serde_json::json!({})))
}
