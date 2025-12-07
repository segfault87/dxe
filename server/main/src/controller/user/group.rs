use actix_web::web;
use dxe_data::queries::identity::{
    delete_group, get_group, get_group_with_members, is_member_of, join_group, leave_group,
    update_group_name, update_group_open, update_group_owner,
};
use dxe_types::GroupId;
use sqlx::SqlitePool;

use crate::config::TimeZoneConfig;
use crate::middleware::datetime_injector::Now;
use crate::models::entities::{Group, GroupWithUsers};
use crate::models::handlers::user::{AmendGroupRequest, AmendGroupResponse, GetGroupResponse};
use crate::models::{Error, IntoView};
use crate::session::UserSession;

pub async fn get(
    now: Now,
    session: UserSession,
    group_id: web::Path<GroupId>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<GetGroupResponse>, Error> {
    let mut connection = database.acquire().await?;

    let mut group = get_group_with_members(&mut connection, &now, group_id.as_ref())
        .await?
        .ok_or(Error::GroupNotFound)?;

    if !group.1.iter().any(|v| v.id != session.user_id) {
        // Hide everyone else owner if you are not the member
        group.1.retain(|v| v.id == group.0.owner_id);
    }

    Ok(web::Json(GetGroupResponse {
        group: GroupWithUsers::convert(group, &timezone_config, &now)?,
    }))
}

pub async fn put(
    now: Now,
    session: UserSession,
    body: web::Json<AmendGroupRequest>,
    group_id: web::Path<GroupId>,
    database: web::Data<SqlitePool>,
    timezone_config: web::Data<TimeZoneConfig>,
) -> Result<web::Json<AmendGroupResponse>, Error> {
    let mut tx = database.begin().await?;

    let group = get_group(&mut tx, &now, group_id.as_ref())
        .await?
        .ok_or(Error::GroupNotFound)?;

    if group.owner_id != session.user_id {
        return Err(Error::GroupNotFound);
    }

    if let Some(new_name) = &body.new_name {
        update_group_name(&mut tx, group_id.as_ref(), new_name).await?;
    }
    if let Some(new_owner) = &body.new_owner {
        if !is_member_of(&mut tx, group_id.as_ref(), new_owner).await? {
            return Err(Error::CannotTransferGroupOwnership);
        }
        update_group_owner(&mut tx, group_id.as_ref(), new_owner).await?;
    }
    if let Some(is_open) = body.is_open {
        update_group_open(&mut tx, group_id.as_ref(), is_open).await?;
    }

    let group = get_group(&mut tx, &now, group_id.as_ref())
        .await?
        .ok_or(Error::GroupNotFound)?;

    tx.commit().await?;

    Ok(web::Json(AmendGroupResponse {
        group: Group::convert(group, &timezone_config, &now)?,
    }))
}

pub async fn delete(
    now: Now,
    session: UserSession,
    group_id: web::Path<GroupId>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let (group, members) = get_group_with_members(&mut tx, &now, group_id.as_ref())
        .await?
        .ok_or(Error::GroupNotFound)?;

    if group.owner_id != session.user_id || members.len() > 1 {
        return Err(Error::CannotDeleteGroup);
    }

    if !delete_group(&mut tx, &now, group_id.as_ref()).await? {
        return Err(Error::CannotDeleteGroup);
    }

    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}

pub async fn membership_put(
    now: Now,
    session: UserSession,
    group_id: web::Path<GroupId>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;

    let group = get_group(&mut tx, &now, group_id.as_ref())
        .await?
        .ok_or(Error::GroupNotFound)?;
    if !group.is_open {
        return Err(Error::GroupIsNotOpen);
    }

    join_group(&mut tx, &now, group_id.as_ref(), &session.user_id).await?;
    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}

pub async fn membership_delete(
    now: Now,
    session: UserSession,
    group_id: web::Path<GroupId>,
    database: web::Data<SqlitePool>,
) -> Result<web::Json<serde_json::Value>, Error> {
    let mut tx = database.begin().await?;
    let group = get_group(&mut tx, &now, &group_id)
        .await?
        .ok_or(Error::GroupNotFound)?;
    if group.owner_id == session.user_id {
        return Err(Error::CannotLeaveGroup);
    }
    let result = leave_group(&mut tx, &group_id, &session.user_id).await?;
    if !result {
        return Err(Error::CannotLeaveGroup);
    }
    tx.commit().await?;

    Ok(web::Json(serde_json::json!({})))
}
