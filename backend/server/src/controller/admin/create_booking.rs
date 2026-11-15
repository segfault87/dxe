use actix_web::web;
use chrono::TimeDelta;
use dxe_data::entities::Identity;
use dxe_data::queries::booking::{
    calculate_booking_attachments_price, create_booking, get_booking,
};
use dxe_data::queries::identity::{get_group_members, get_identity};
use dxe_data::queries::payment::{create_cash_transaction, get_cash_transaction};
use dxe_data::queries::user::get_user_by_id;
use dxe_types::ProductId;
use sqlx::SqlitePool;

use crate::config::{BookingConfig, TimeZoneConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{Booking, CashTransaction};
use crate::models::handlers::admin::{CreateBookingRequest, CreateBookingResponse};
use crate::models::{Error, IntoView};
use crate::services::calendar::CalendarService;
use crate::services::messaging::MessagingService;
use crate::utils::datetime::truncate_time;

pub async fn post(
    now: Now,
    body: web::Json<CreateBookingRequest>,
    database: web::Data<SqlitePool>,
    booking_config: web::Data<BookingConfig>,
    timezone_config: web::Data<TimeZoneConfig>,
    messaging_service: web::Data<MessagingService>,
    calendar_service: web::Data<Option<CalendarService>>,
) -> Result<web::Json<CreateBookingResponse>, Error> {
    if body.desired_hours <= 0 {
        return Err(Error::InvalidTimeRange);
    }

    let mut tx = database.begin().await?;

    let time_from = truncate_time(body.time_from).to_utc();
    let time_to = time_from + TimeDelta::hours(body.desired_hours);

    let holder = get_user_by_id(&mut tx, &body.holder_id, &now)
        .await?
        .ok_or(Error::UserNotFound)?;

    let customer = get_identity(&mut tx, &now, &body.customer_id)
        .await?
        .ok_or(Error::UserNotFound)?;

    let customers = match &customer {
        Identity::User(u) => vec![u.clone()],
        Identity::Group(g) => get_group_members(&mut tx, &g.id).await?,
    };

    let booking_id = create_booking(
        &mut tx,
        &now,
        &body.unit_id,
        &holder.id,
        &body.customer_id,
        &time_from,
        &time_to,
        body.skip_payment,
        &body.attachment_ids,
    )
    .await?;

    let product_id = ProductId::from(booking_id);

    if !body.skip_payment {
        let price = match body.price {
            Some(price) if price < 0 => return Err(Error::MissingField("price")),
            Some(price) => price,
            None => {
                booking_config
                    .calculate_price(&body.unit_id, time_from, time_to)
                    .map_err(|_| Error::UnitNotFound)?
                    + calculate_booking_attachments_price(&mut tx, &now, &body.attachment_ids)
                        .await?
            }
        };

        let depositor_name = body
            .depositor_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or(holder.name.as_str());

        create_cash_transaction(&mut tx, &now, &product_id, depositor_name, price).await?;
    }

    let booking = get_booking(&mut tx, &booking_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    let cash_tx = get_cash_transaction(&mut tx, &product_id).await?;

    if body.skip_payment {
        messaging_service
            .send_confirmation(&mut tx, booking.clone())
            .await?;
    }

    tx.commit().await?;

    if let Some(calendar_service) = calendar_service.as_ref()
        && let Err(e) = calendar_service
            .register_booking(&booking, &customers)
            .await
    {
        log::error!("Failed to register event on calendar: {e}");
    }

    Ok(web::Json(CreateBookingResponse {
        booking: Booking::convert(booking, &timezone_config, &now)?
            .finish(booking_config.as_ref(), &now),
        cash_transaction: cash_tx
            .map(|v| CashTransaction::convert(v, &timezone_config, &now))
            .transpose()?,
    }))
}
