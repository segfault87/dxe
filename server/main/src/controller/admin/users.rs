use actix_web::web;
use dxe_data::queries::user::get_users;
use sqlx::SqlitePool;

use crate::config::TimeZoneConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::entities::SelfUser;
use crate::models::handlers::admin::GetUsersResponse;
use crate::models::{Error, IntoView};

pub async fn get(
    now: Now,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetUsersResponse>, Error> {
    let mut connection = database.acquire().await?;

    let users = get_users(&mut connection).await?;

    Ok(web::Json(GetUsersResponse {
        users: users
            .into_iter()
            .map(|v| SelfUser::convert(v, &timezone_config, &now))
            .collect::<Result<_, _>>()?,
    }))
}
