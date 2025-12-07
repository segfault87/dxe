use actix_web::web;
use chrono::TimeDelta;
use dxe_data::queries::booking::get_adhoc_parkings;
use dxe_s2s_shared::entities::AdhocParking;
use dxe_s2s_shared::handlers::GetAdhocParkingsResponse;
use sqlx::SqlitePool;

use crate::middleware::coordinator_verifier::CoordinatorContext;
use crate::middleware::datetime_injector::Now;
use crate::models::Error;

pub async fn get(
    now: Now,
    context: CoordinatorContext,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<GetAdhocParkingsResponse>, Error> {
    let range_from = *now;
    let range_to = range_from + TimeDelta::days(1);

    let mut connection = database.acquire().await?;

    let parkings = get_adhoc_parkings(
        &mut connection,
        &context.space_id,
        Some(range_from),
        Some(range_to),
    )
    .await?;

    Ok(web::Json(GetAdhocParkingsResponse {
        parkings: parkings
            .into_iter()
            .map(|v| AdhocParking {
                id: v.id,
                time_from: v.time_from.into(),
                time_to: v.time_to.into(),
                license_plate_number: v.license_plate_number,
            })
            .collect(),
    }))
}
