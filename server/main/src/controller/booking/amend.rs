use actix_web::web;
use chrono::Utc;
use dxe_data::entities::Identity;
use dxe_data::queries::booking::{get_booking_with_user_id, update_booking_customer};
use dxe_data::queries::identity::is_member_of;
use dxe_data::types::{BookingId, GroupId};
use sqlx::SqlitePool;

use crate::config::TimeZoneConfig;
use crate::models::entities::Booking;
use crate::models::handlers::booking::{AmendBookingRequest, AmendBookingResponse};
use crate::models::{Error, IntoView};
use crate::session::UserSession;

pub async fn put(
    session: UserSession,
    booking_id: web::Path<BookingId>,
    body: web::Json<AmendBookingRequest>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<AmendBookingResponse>, Error> {
    let now = Utc::now();

    let mut tx = database.begin().await?;

    let booking = get_booking_with_user_id(&mut tx, booking_id.as_ref(), &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

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

    let booking = get_booking_with_user_id(&mut tx, booking_id.as_ref(), &session.user_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    tx.commit().await?;

    Ok(web::Json(AmendBookingResponse {
        booking: Booking::convert(booking, timezone_config.as_ref(), &now),
    }))
}
