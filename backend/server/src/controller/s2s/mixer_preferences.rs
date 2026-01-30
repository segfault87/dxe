use actix_web::web;
use dxe_data::queries::prefs::{create_or_update_mixer_config, get_mixer_config};
use dxe_s2s_shared::handlers::{
    GetMixerConfigQuery, GetMixerConfigResponse, UpdateMixerConfigRequest,
};
use sqlx::SqlitePool;

use crate::middleware::datetime_injector::Now;
use crate::models::Error;

pub async fn post(
    now: Now,
    body: web::Json<UpdateMixerConfigRequest>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let _ =
        create_or_update_mixer_config(&mut *tx, &now, body.identity_id, &body.unit_id, &body.prefs)
            .await?;

    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}

pub async fn get(
    query: web::Query<GetMixerConfigQuery>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<GetMixerConfigResponse>, Error> {
    let mut conn = database.acquire().await?;

    let result = get_mixer_config(&mut *conn, query.identity_id, &query.unit_id).await?;

    Ok(web::Json(GetMixerConfigResponse {
        prefs: result.map(|v| v.data.0),
    }))
}
