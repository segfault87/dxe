use actix_web::web;
use dxe_data::queries::identity::{
    create_group, get_group_with_members, get_groups_associated_with_members,
};
use sqlx::SqlitePool;

use crate::config::TimeZoneConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::entities::GroupWithUsers;
use crate::models::handlers::user::{CreateGroupRequest, CreateGroupResponse, ListGroupsResponse};
use crate::models::{Error, IntoView};
use crate::session::UserSession;

pub async fn get(
    now: Now,
    session: UserSession,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<ListGroupsResponse>, Error> {
    let mut connection = database.acquire().await?;

    let mut groups =
        get_groups_associated_with_members(&mut connection, &now, &session.user_id).await?;
    groups.sort_by(|a, b| a.0.name.cmp(&b.0.name));

    Ok(web::Json(ListGroupsResponse {
        groups: groups
            .into_iter()
            .map(|v| GroupWithUsers::convert(v, &timezone_config, &now))
            .collect::<Result<Vec<_>, _>>()?,
    }))
}

pub async fn post(
    now: Now,
    session: UserSession,
    body: web::Json<CreateGroupRequest>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<CreateGroupResponse>, Error> {
    let mut tx = database.begin().await?;

    let group_id = create_group(&mut tx, &now, &session.user_id, &body.name, true).await?;
    let group = get_group_with_members(&mut tx, &now, &group_id)
        .await?
        .ok_or(Error::UserNotFound)?;

    tx.commit().await?;

    Ok(web::Json(CreateGroupResponse {
        group: GroupWithUsers::convert(group, &timezone_config, &now)?,
    }))
}
