use actix_web::web;
use chrono::TimeDelta;
use dxe_data::queries::booking::{create_adhoc_parking, delete_adhoc_parking, get_adhoc_parkings};
use dxe_types::AdhocParkingId;
use sqlx::SqlitePool;

use crate::config::TimeZoneConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::entities::AdhocParking;
use crate::models::handlers::admin::{
    CreateAdhocParkingRequest, GetAdhocParkingsQuery, GetAdhocParkingsResponse,
};
use crate::models::{Error, IntoView};
use crate::session::UserSession;
use crate::utils::datetime::truncate_time;

pub async fn get(
    now: Now,
    query: web::Query<GetAdhocParkingsQuery>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetAdhocParkingsResponse>, Error> {
    let mut connection = database.acquire().await?;

    let parkings = get_adhoc_parkings(&mut connection, &query.space_id, Some(*now), None).await?;

    Ok(web::Json(GetAdhocParkingsResponse {
        parkings: parkings
            .into_iter()
            .map(|v| AdhocParking::convert(v, &timezone_config, &now))
            .collect::<Result<_, _>>()?,
    }))
}

pub async fn post(
    now: Now,
    _session: UserSession,
    body: web::Json<CreateAdhocParkingRequest>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let time_from = truncate_time(body.time_from).to_utc();
    let time_to = time_from + TimeDelta::hours(body.desired_hours);

    let _ = create_adhoc_parking(
        &mut tx,
        &now,
        &body.space_id,
        &time_from,
        &time_to,
        &body.license_plate_number,
    )
    .await?;

    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}

pub async fn delete(
    adhoc_parking_id: web::Path<AdhocParkingId>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    delete_adhoc_parking(&mut tx, *adhoc_parking_id).await?;

    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}
