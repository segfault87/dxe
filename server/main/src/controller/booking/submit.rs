use actix_web::web;
use chrono::{TimeDelta, Utc};
use dxe_data::entities::Identity;
use dxe_data::queries::booking::{
    create_booking, create_cash_payment_status, get_booking_with_user_id, get_cash_payment_status,
    is_booking_available,
};
use dxe_data::queries::identity::{get_group_members, get_identity, is_member_of};
use dxe_data::queries::unit::is_unit_enabled;
use dxe_data::queries::user::update_user_cash_payment_depositor_name;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::models::entities::{Booking, BookingCashPaymentStatus};
use crate::models::handlers::booking::{SubmitBookingRequest, SubmitBookingResponse};
use crate::models::{Error, IntoView};
use crate::services::calendar::CalendarService;
use crate::services::telemetry::{NotificationSender, Priority};
use crate::session::UserSession;
use crate::utils::datetime::truncate_time;

pub async fn post(
    session: UserSession,
    body: web::Json<SubmitBookingRequest>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
    notification_sender: web::Data<NotificationSender>,
    calendar_service: web::Data<Option<CalendarService>>,
) -> Result<web::Json<SubmitBookingResponse>, Error> {
    let now = Utc::now();

    let mut tx = database.begin().await?;

    if is_unit_enabled(&mut tx, &body.unit_id).await? != Some(true) {
        return Err(Error::UnitNotFound);
    }

    if body.desired_hours > booking_config.max_booking_hours {
        return Err(Error::InvalidTimeRange);
    }

    let time_from = truncate_time(body.time_from).to_utc();
    let time_to = time_from + TimeDelta::hours(body.desired_hours);

    if !is_booking_available(&mut tx, &now, &body.unit_id, &time_from, &time_to).await? {
        return Err(Error::TimeRangeOccupied);
    }

    let identity = get_identity(&mut tx, &now, &body.identity_id)
        .await?
        .ok_or(Error::UserNotFound)?;

    let customers = match &identity {
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

    let booking_id = create_booking(
        &mut tx,
        &now,
        &body.unit_id,
        &session.user_id,
        &body.identity_id,
        &time_from,
        &time_to,
    )
    .await?;

    let price = booking_config
        .calculate_price(&body.unit_id, time_from, time_to)
        .map_err(|_| Error::UnitNotFound)?;

    update_user_cash_payment_depositor_name(
        &mut tx,
        &session.user_id,
        Some(body.depositor_name.as_str()),
    )
    .await?;

    create_cash_payment_status(
        &mut tx,
        &now,
        &booking_id,
        body.depositor_name.as_str(),
        price,
    )
    .await?;

    let booking = get_booking_with_user_id(&mut tx, &booking_id, &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    let cash_payment_status = get_cash_payment_status(&mut tx, &booking_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    tx.commit().await?;

    if let Some(calendar_service) = calendar_service.as_ref() {
        if let Err(e) = calendar_service
            .register_booking(&timezone_config, &booking, &customers)
            .await
        {
            log::error!("Failed to register event on calendar: {e}");
        }
    }

    notification_sender.enqueue(
        Priority::High,
        format!(
            "New booking by {}: {} ({} hours)",
            identity.name(),
            timezone_config.convert(time_from),
            body.desired_hours
        ),
    );

    Ok(web::Json(SubmitBookingResponse {
        booking: Booking::convert(booking, &timezone_config, &now)?
            .finish(booking_config.as_ref(), &now),
        cash_payment_status: BookingCashPaymentStatus::convert(
            cash_payment_status,
            &timezone_config,
            &now,
        )?,
    }))
}
