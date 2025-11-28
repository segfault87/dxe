use actix_web::web;
use chrono::Utc;
use dxe_data::queries::identity::get_all_groups_associated_with_members;
use sqlx::SqlitePool;

use crate::config::TimeZoneConfig;
use crate::models::entities::GroupWithUsers;
use crate::models::handlers::admin::GetGroupsResponse;
use crate::models::{Error, IntoView};

pub async fn get(
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetGroupsResponse>, Error> {
    let now = Utc::now();

    let mut connection = database.acquire().await?;

    let groups = get_all_groups_associated_with_members(&mut connection, &now).await?;

    Ok(web::Json(GetGroupsResponse {
        groups: groups
            .into_iter()
            .map(|v| GroupWithUsers::convert(v, &timezone_config, &now))
            .collect::<Result<_, _>>()?,
    }))
}
