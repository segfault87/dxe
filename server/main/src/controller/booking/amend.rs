use actix_web::web;
use chrono::TimeDelta;
use dxe_data::entities::Identity;
use dxe_data::queries::booking::{
    create_booking_amendment, get_booking_with_user_id, update_booking_customer,
    update_booking_time,
};
use dxe_data::queries::identity::{get_group_members, is_member_of};
use dxe_data::queries::payment::create_toss_payments_transaction;
use dxe_data::utils::is_in_effect;
use dxe_types::{BookingId, ForeignPaymentId, GroupId, ProductId};
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::Booking;
use crate::models::handlers::booking::{AmendBookingRequest, AmendBookingResponse};
use crate::models::{Error, IntoView};
use crate::services::calendar::CalendarService;
use crate::services::messaging::biztalk::BiztalkSender;
use crate::services::telemetry::{NotificationSender, Priority};
use crate::session::UserSession;
use crate::utils::messaging::send_amend_notification;

pub async fn put(
    now: Now,
    session: UserSession,
    booking_id: web::Path<BookingId>,
    body: web::Json<AmendBookingRequest>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
    calendar_service: web::Data<Option<CalendarService>>,
    notification_sender: web::Data<NotificationSender>,
    biztalk_sender: web::Data<Option<BiztalkSender>>,
) -> Result<web::Json<AmendBookingResponse>, Error> {
    let mut tx = database.begin().await?;

    let booking = get_booking_with_user_id(&mut tx, booking_id.as_ref(), &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    if booking.holder.id != session.user_id {
        return Err(Error::BookingNotFound);
    }

    if is_in_effect(&booking.canceled_at, &now) {
        return Err(Error::BookingNotFound);
    }

    if let Some(new_identity_id) = &body.new_identity_id {
        if matches!(booking.customer, Identity::Group(_)) {
            return Err(Error::BookingNotAssignableToGroup);
        }

        let group_id = GroupId::from(*new_identity_id);

        if !is_member_of(&mut tx, &group_id, &session.user_id).await? {
            return Err(Error::UserNotMemberOf);
        }

        update_booking_customer(&mut tx, booking_id.as_ref(), new_identity_id).await?;
    }

    let mut foreign_payment_id = None;

    let current_hours = (booking.time_to - booking.time_from).num_hours();
    if let Some(additional_hours) = body.additional_hours
        && additional_hours > 0
    {
        if *now >= booking.time_from {
            return Err(Error::OngoingBookingNotModifiable);
        }

        let total_hours = additional_hours + current_hours;
        if total_hours > booking_config.max_booking_hours {
            return Err(Error::InvalidTimeRange);
        }

        let desired_time_from = booking.time_from;
        let desired_time_to = desired_time_from + TimeDelta::hours(total_hours);

        let price = booking_config
            .calculate_additive_price(&booking.unit_id, additional_hours)
            .map_err(|_| Error::UnitNotFound)?;

        let booking_amendment_id = create_booking_amendment(
            &mut tx,
            &now,
            &booking_id,
            &desired_time_from,
            &desired_time_to,
            false,
        )
        .await?;

        let product_id = ProductId::from(booking_amendment_id);
        let foreign_payment_id_inner = ForeignPaymentId::generate();

        let _ = create_toss_payments_transaction(
            &mut tx,
            &now,
            &foreign_payment_id_inner,
            &session.user_id,
            None,
            Some(&product_id),
            price,
        )
        .await?;

        foreign_payment_id = Some(foreign_payment_id_inner);
    } else if let Some(new_time_from) = body.new_time_from {
        if *now >= booking.time_from {
            return Err(Error::OngoingBookingNotModifiable);
        }

        let total_hours = current_hours + body.additional_hours.unwrap_or(0);
        if total_hours > booking_config.max_booking_hours {
            return Err(Error::InvalidTimeRange);
        }

        let desired_time_from = new_time_from.to_utc();
        let desired_time_to = desired_time_from + TimeDelta::hours(total_hours);

        let _ = create_booking_amendment(
            &mut tx,
            &now,
            &booking_id,
            &desired_time_from,
            &desired_time_to,
            true,
        )
        .await?;

        if update_booking_time(
            &mut tx,
            &now,
            &booking_id,
            &desired_time_from,
            &desired_time_to,
        )
        .await?
        {
            if let Err(e) = send_amend_notification(
                &biztalk_sender,
                &mut tx,
                &timezone_config,
                &booking,
                &desired_time_from,
                &desired_time_to,
            )
            .await
            {
                log::warn!("Could not send amend notification to customers: {e}");
            }

            notification_sender.enqueue(
                Priority::High,
                format!(
                    "Booking by {} moved from {} - {} to {} - {}",
                    booking.customer.name(),
                    timezone_config.convert(booking.time_from),
                    timezone_config.convert(booking.time_to),
                    timezone_config.convert(desired_time_from),
                    timezone_config.convert(desired_time_to),
                ),
            );

            if let Some(calendar_service) = calendar_service.as_ref() {
                let users = match &booking.customer {
                    Identity::User(u) => {
                        vec![u.clone()]
                    }
                    Identity::Group(g) => get_group_members(&mut tx, &g.id).await?,
                };

                if let Err(e) = calendar_service.delete_booking(&booking_id).await {
                    log::warn!("Could not remove previous booking from calendar: {e}");
                }
                if let Err(e) = calendar_service
                    .register_booking(&timezone_config, &booking, &users)
                    .await
                {
                    log::warn!("Could not adding new booking entry to calendar: {e}");
                }
            }
        }
    }

    let booking = get_booking_with_user_id(&mut tx, booking_id.as_ref(), &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    tx.commit().await?;

    Ok(web::Json(AmendBookingResponse {
        booking: Booking::convert(booking, timezone_config.as_ref(), &now)?,
        foreign_payment_id,
    }))
}
