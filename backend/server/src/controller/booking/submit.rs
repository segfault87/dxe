use actix_web::web;
use chrono::TimeDelta;
use dxe_data::entities::Identity;
use dxe_data::queries::booking::{create_booking, get_booking_with_user_id};
use dxe_data::queries::identity::{get_group_members, get_identity, is_member_of};
use dxe_data::queries::payment::{create_cash_transaction, get_cash_transaction};
use dxe_data::queries::unit::is_unit_enabled;
use dxe_data::queries::user::update_user_cash_payment_depositor_name;
use dxe_types::ProductId;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{Booking, CashTransaction};
use crate::models::handlers::booking::{SubmitBookingRequest, SubmitBookingResponse};
use crate::models::{Error, IntoView};
use crate::services::calendar::CalendarService;
use crate::services::notification::{NotificationSender, Priority};
use crate::session::UserSession;
use crate::utils::datetime::truncate_time;

pub async fn post(
    now: Now,
    session: UserSession,
    body: web::Json<SubmitBookingRequest>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
    notification_sender: web::Data<NotificationSender>,
    calendar_service: web::Data<Option<CalendarService>>,
) -> Result<web::Json<SubmitBookingResponse>, Error> {
    let mut tx = database.begin().await?;

    if is_unit_enabled(&mut tx, &body.unit_id).await? != Some(true) {
        return Err(Error::UnitNotFound);
    }

    if body.desired_hours > booking_config.max_booking_hours {
        return Err(Error::InvalidTimeRange);
    }

    let time_from = truncate_time(body.time_from).to_utc();
    let time_to = time_from + TimeDelta::hours(body.desired_hours);

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
        false,
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

    let product_id = ProductId::from(booking_id);

    create_cash_transaction(
        &mut tx,
        &now,
        &product_id,
        body.depositor_name.as_str(),
        price,
    )
    .await?;

    let booking = get_booking_with_user_id(&mut tx, &booking_id, &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    let cash_tx = get_cash_transaction(&mut tx, &product_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    tx.commit().await?;

    if let Some(calendar_service) = calendar_service.as_ref()
        && let Err(e) = calendar_service
            .register_booking(&booking, &customers)
            .await
    {
        log::error!("Failed to register event on calendar: {e}");
    }

    notification_sender.enqueue(
        Priority::High,
        format!(
            "New booking request by {}: {} ({} hours)",
            identity.name(),
            timezone_config.convert(time_from),
            body.desired_hours
        ),
    );

    Ok(web::Json(SubmitBookingResponse {
        booking: Booking::convert(booking, &timezone_config, &now)?
            .finish(booking_config.as_ref(), &now),
        cash_transaction: CashTransaction::convert(cash_tx, &timezone_config, &now)?,
    }))
}
