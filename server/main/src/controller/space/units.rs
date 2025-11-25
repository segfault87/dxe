use actix_web::web;
use dxe_data::queries::unit::get_units_by_space_id;
use dxe_s2s_shared::entities::Unit;
use dxe_s2s_shared::handlers::GetUnitsResponse;
use sqlx::SqlitePool;

use crate::middleware::coordinator_verifier::CoordinatorContext;
use crate::models::Error;

pub async fn get(
    context: CoordinatorContext,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<GetUnitsResponse>, Error> {
    let mut connection = database.begin().await?;

    let units = get_units_by_space_id(&mut *connection, &context.space_id).await?;

    Ok(web::Json(GetUnitsResponse {
        units: units
            .into_iter()
            .map(|v| Unit {
                id: v.id,
                enabled: v.enabled,
                space_id: v.space_id,
            })
            .collect(),
    }))
}
