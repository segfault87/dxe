use actix_web::web;
use dxe_data::queries::identity::get_identity;
use dxe_data::queries::payment::{
    cancel_adhoc_payment, create_adhoc_payment, get_adhoc_payment, get_adhoc_payments,
};
use dxe_types::AdHocPaymentId;
use sqlx::SqlitePool;

use crate::config::{TimeZoneConfig, UrlConfig};
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{AdHocPayment, AdHocPaymentWithUrl};
use crate::models::handlers::admin::{
    CreateAdHocPaymentRequest, CreateAdHocPaymentResponse, GetAdHocPaymentsQuery,
    GetAdHocPaymentsResponse,
};
use crate::models::{Error, IntoView};
use crate::session::UserSession;

const DEFAULT_LIMIT: i64 = 50;
const MAX_NOTE_LENGTH: usize = 200;

fn payment_url(url_config: &UrlConfig, id: &AdHocPaymentId) -> url::Url {
    let mut url = url_config.base_url.clone();
    url.set_path(&format!("payment/{id}"));
    url
}

fn with_url(
    entity: dxe_data::entities::AdHocPayment,
    timezone_config: &TimeZoneConfig,
    url_config: &UrlConfig,
    now: &Now,
) -> Result<AdHocPaymentWithUrl, Error> {
    let url = payment_url(url_config, &entity.id);

    Ok(AdHocPaymentWithUrl {
        payment: AdHocPayment::convert(entity, timezone_config, now)?,
        url,
    })
}

pub async fn get(
    now: Now,
    query: web::Query<GetAdHocPaymentsQuery>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
    url_config: web::Data<UrlConfig>,
) -> Result<web::Json<GetAdHocPaymentsResponse>, Error> {
    let mut connection = database.acquire().await?;

    let payments = get_adhoc_payments(
        &mut connection,
        &now,
        query.limit.unwrap_or(DEFAULT_LIMIT),
        query.offset.unwrap_or(0),
    )
    .await?;

    Ok(web::Json(GetAdHocPaymentsResponse {
        payments: payments
            .into_iter()
            .map(|v| with_url(v, &timezone_config, &url_config, &now))
            .collect::<Result<_, _>>()?,
    }))
}

pub async fn post(
    now: Now,
    session: UserSession,
    body: web::Json<CreateAdHocPaymentRequest>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
    url_config: web::Data<UrlConfig>,
) -> Result<web::Json<CreateAdHocPaymentResponse>, Error> {
    if body.amount <= 0 {
        return Err(Error::MissingField("amount"));
    }

    let note = body.note.trim();
    if note.is_empty() || note.chars().count() > MAX_NOTE_LENGTH {
        return Err(Error::MissingField("note"));
    }

    let mut tx = database.begin().await?;

    let recipient = get_identity(&mut tx, &now, &body.recipient_id)
        .await?
        .ok_or(Error::UserNotFound)?;

    let id = create_adhoc_payment(
        &mut tx,
        &now,
        &session.user_id,
        &recipient.id(),
        body.amount,
        note,
    )
    .await?;

    let payment = get_adhoc_payment(&mut tx, &now, &id)
        .await?
        .ok_or(Error::AdHocPaymentNotFound)?;

    tx.commit().await?;

    Ok(web::Json(CreateAdHocPaymentResponse {
        payment: with_url(payment, &timezone_config, &url_config, &now)?,
    }))
}

pub async fn delete(
    now: Now,
    id: web::Path<AdHocPaymentId>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    if get_adhoc_payment(&mut tx, &now, &id).await?.is_none() {
        return Err(Error::AdHocPaymentNotFound);
    }

    if !cancel_adhoc_payment(&mut tx, &now, &id).await? {
        return Err(Error::AdHocPaymentNotPayable);
    }

    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}
