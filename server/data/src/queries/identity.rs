use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::SqliteConnection;

use crate::Error;
use crate::entities::{Group, Identity, IdentityDiscriminator, User};
use crate::types::{GroupId, IdentityId, IdentityProvider, UserId};

pub async fn get_identity(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    identity_id: &IdentityId,
) -> Result<Option<Identity>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT
            i.discriminator as "i_discriminator: IdentityDiscriminator",
            u.id as "u_id: Option<UserId>",
            u.provider as "u_provider: Option<IdentityProvider>",
            u.foreign_id as "u_foreign_id: Option<String>",
            u.name as "u_name: Option<String>",
            u.created_at as "u_created_at: Option<DateTime<Utc>>",
            u.deactivated_at as "u_deactivated_at: DateTime<Utc>",
            u.license_plate_number as "u_license_plate_number",
            g.id as "g_id: Option<GroupId>",
            g.name as "g_name: Option<String>",
            g.owner_id AS "g_owner_id: Option<UserId>",
            g.is_open AS "g_is_open: Option<bool>",
            g.created_at AS "g_created_at: Option<DateTime<Utc>>",
            g.deleted_at AS "g_deleted_at: DateTime<Utc>"
        FROM identity "i"
        LEFT OUTER JOIN user "u" ON i.discriminator = 'user' AND i.id = u.id
        LEFT OUTER JOIN "group" "g" ON i.discriminator = 'group' AND i.id = g.id
        WHERE
            i.id = ?1 AND
            (g.deleted_at IS NULL OR g.deleted_at < ?2) AND
            (u.deactivated_at IS NULL OR u.deactivated_at < ?2)
        "#,
        identity_id,
        now
    )
    .fetch_optional(&mut *connection)
    .await?;

    if let Some(result) = result {
        match result.i_discriminator {
            IdentityDiscriminator::User => Ok(Some(Identity::User(User {
                id: result.u_id.ok_or(Error::MissingField("u.id"))?,
                provider: result.u_provider.ok_or(Error::MissingField("u.provider"))?,
                foreign_id: result
                    .u_foreign_id
                    .ok_or(Error::MissingField("u.foreign_id"))?,
                name: result.u_name.ok_or(Error::MissingField("u.name"))?,
                created_at: result
                    .u_created_at
                    .ok_or(Error::MissingField("u.created_at"))?,
                deactivated_at: result.u_deactivated_at,
                license_plate_number: result.u_license_plate_number,
            }))),
            IdentityDiscriminator::Group => Ok(Some(Identity::Group(Group {
                id: result.g_id.ok_or(Error::MissingField("g.id"))?,
                name: result.g_name.ok_or(Error::MissingField("g.name"))?,
                owner_id: result.g_owner_id.ok_or(Error::MissingField("g.owner_id"))?,
                is_open: result.g_is_open.ok_or(Error::MissingField("g.is_open"))?,
                created_at: result
                    .g_created_at
                    .ok_or(Error::MissingField("g.created_at"))?,
                deleted_at: result.g_deleted_at,
            }))),
        }
    } else {
        Ok(None)
    }
}

pub async fn get_group(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    group_id: &GroupId,
) -> Result<Option<Group>, Error> {
    Ok(sqlx::query_as!(
        Group,
        r#"
        SELECT
            id AS "id: _",
            name,
            owner_id AS "owner_id: _",
            is_open AS "is_open: _",
            created_at AS "created_at: _",
            deleted_at AS "deleted_at: _"
        FROM "group"
        WHERE
            id = ?1 AND
            (deleted_at IS NULL OR deleted_at < ?2)
        "#,
        group_id,
        now,
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn get_group_with_members(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    group_id: &GroupId,
) -> Result<Option<(Group, Vec<User>)>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT
            g.id AS "g_id: GroupId",
            g.name AS "g_name",
            g.owner_id AS "g_owner_id: UserId",
            g.is_open AS "g_is_open: bool",
            g.created_at AS "g_created_at: DateTime<Utc>",
            g.deleted_at AS "g_deleted_at: DateTime<Utc>",
            u.id as "u_id: UserId",
            u.provider as "u_provider: IdentityProvider",
            u.foreign_id as "u_foreign_id",
            u.name as "u_name",
            u.created_at as "u_created_at: DateTime<Utc>",
            u.deactivated_at as "u_deactivated_at: DateTime<Utc>",
            u.license_plate_number as "u_license_plate_number"
        FROM "group" "g"
        JOIN group_association "ga" ON g.id = ga.group_id
        JOIN user "u" ON ga.user_id = u.id
        WHERE
            g.id = ?1 AND
            (g.deleted_at IS NULL OR g.deleted_at < ?2)
        "#,
        group_id,
        now
    )
    .fetch_all(&mut *connection)
    .await?;

    let mut group = None;
    let mut users = vec![];
    for i in result {
        if group.is_none() {
            group = Some(Group {
                id: i.g_id,
                name: i.g_name,
                owner_id: i.g_owner_id,
                is_open: i.g_is_open,
                created_at: i.g_created_at,
                deleted_at: i.g_deleted_at,
            });
        }

        users.push(User {
            id: i.u_id,
            provider: i.u_provider,
            foreign_id: i.u_foreign_id,
            name: i.u_name,
            created_at: i.u_created_at,
            deactivated_at: i.u_deactivated_at,
            license_plate_number: i.u_license_plate_number,
        });
    }

    if let Some(group) = group {
        Ok(Some((group, users)))
    } else {
        Ok(None)
    }
}

pub async fn get_groups_by_owner(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    owner_id: &UserId,
) -> Result<Vec<Group>, Error> {
    Ok(sqlx::query_as!(
        Group,
        r#"
        SELECT
            id AS "id: _",
            name,
            owner_id AS "owner_id: _",
            is_open AS "is_open: _",
            created_at AS "created_at: _",
            deleted_at AS "deleted_at: _"
        FROM "group"
        WHERE
            owner_id=?1 AND
            (deleted_at IS NULL OR deleted_at < ?2)
        "#,
        owner_id,
        now,
    )
    .fetch_all(&mut *connection)
    .await?)
}

pub async fn get_all_groups_associated_with_members(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
) -> Result<Vec<(Group, Vec<User>)>, Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            g.id AS "g_id: GroupId",
            g.name AS "g_name",
            g.owner_id AS "g_owner_id: UserId",
            g.is_open AS "g_is_open: bool",
            g.created_at AS "g_created_at: DateTime<Utc>",
            g.deleted_at AS "g_deleted_at: DateTime<Utc>",
            u.id AS "u_id: UserId",
            u.provider AS "u_provider: IdentityProvider",
            u.foreign_id AS "u_foreign_id",
            u.name AS "u_name",
            u.created_at AS "u_created_at: DateTime<Utc>",
            u.deactivated_at AS "u_deactivated_at: DateTime<Utc>",
            u.license_plate_number AS "u_license_plate_number"
        FROM "group" "g"
        JOIN group_association "ga" ON g.id = ga.group_id
        JOIN user "u" ON ga.user_id = u.id
        WHERE
            (g.deleted_at IS NULL OR g.deleted_at < ?1)
        "#,
        now,
    )
    .fetch_all(&mut *connection)
    .await?;

    let mut result = HashMap::new();
    for row in rows {
        let (_, users) = result.entry(row.g_id).or_insert_with(|| {
            (
                Group {
                    id: row.g_id,
                    name: row.g_name,
                    owner_id: row.g_owner_id,
                    is_open: row.g_is_open,
                    created_at: row.g_created_at,
                    deleted_at: row.g_deleted_at,
                },
                Vec::new(),
            )
        });

        users.push(User {
            id: row.u_id,
            provider: row.u_provider,
            foreign_id: row.u_foreign_id,
            name: row.u_name,
            created_at: row.u_created_at,
            deactivated_at: row.u_deactivated_at,
            license_plate_number: row.u_license_plate_number,
        });
    }

    Ok(result.into_values().collect())
}

pub async fn get_groups_associated_with_members(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    user_id: &UserId,
) -> Result<Vec<(Group, Vec<User>)>, Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            g.id AS "g_id: GroupId",
            g.name AS "g_name",
            g.owner_id AS "g_owner_id: UserId",
            g.is_open AS "g_is_open: bool",
            g.created_at AS "g_created_at: DateTime<Utc>",
            g.deleted_at AS "g_deleted_at: DateTime<Utc>",
            u.id AS "u_id: UserId",
            u.provider AS "u_provider: IdentityProvider",
            u.foreign_id AS "u_foreign_id",
            u.name AS "u_name",
            u.created_at AS "u_created_at: DateTime<Utc>",
            u.deactivated_at AS "u_deactivated_at: DateTime<Utc>",
            u.license_plate_number AS "u_license_plate_number"
        FROM "group" "g"
        JOIN group_association "ga" ON g.id = ga.group_id
        JOIN user "u" ON ga.user_id = u.id
        WHERE
            (g.deleted_at IS NULL OR g.deleted_at < ?2) AND
            EXISTS (
                SELECT
                    user_id
                FROM
                    group_association
                WHERE
                    group_association.group_id = g.id AND
                    group_association.user_id = ?1
            )
        ORDER BY g.created_at DESC
        "#,
        user_id,
        now,
    )
    .fetch_all(&mut *connection)
    .await?;

    let mut result = HashMap::new();
    for row in rows {
        let (_, users) = result.entry(row.g_id).or_insert_with(|| {
            (
                Group {
                    id: row.g_id,
                    name: row.g_name,
                    owner_id: row.g_owner_id,
                    is_open: row.g_is_open,
                    created_at: row.g_created_at,
                    deleted_at: row.g_deleted_at,
                },
                Vec::new(),
            )
        });

        users.push(User {
            id: row.u_id,
            provider: row.u_provider,
            foreign_id: row.u_foreign_id,
            name: row.u_name,
            created_at: row.u_created_at,
            deactivated_at: row.u_deactivated_at,
            license_plate_number: row.u_license_plate_number,
        });
    }

    Ok(result.into_values().collect())
}

pub async fn create_group(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    user_id: &UserId,
    name: &String,
    is_open: bool,
) -> Result<GroupId, Error> {
    let identity_id = IdentityId::generate();
    let group_id: GroupId = identity_id.into();

    let _ = sqlx::query!(
        r#"
        INSERT INTO "group"(id, name, owner_id, is_open, created_at)
        VALUES(?1, ?2, ?3, ?4, ?5)
        "#,
        group_id,
        name,
        user_id,
        is_open,
        now
    )
    .execute(&mut *connection)
    .await?;

    let _ = sqlx::query!(
        r#"
        INSERT INTO identity(id, discriminator)
        VALUES(?1, 'group')
        "#,
        identity_id
    )
    .execute(&mut *connection)
    .await?;

    let _ = sqlx::query!(
        r#"
        INSERT INTO group_association(group_id, user_id, joined_at)
        VALUES(?1, ?2, ?3)
        "#,
        group_id,
        user_id,
        now
    )
    .execute(&mut *connection)
    .await?;

    Ok(group_id)
}

pub async fn update_group_name(
    connection: &mut SqliteConnection,
    group_id: &GroupId,
    name: &String,
) -> Result<(), Error> {
    let _ = sqlx::query!(
        r#"
        UPDATE "group"
        SET name=?1
        WHERE id=?2
        "#,
        name,
        group_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn update_group_open(
    connection: &mut SqliteConnection,
    group_id: &GroupId,
    is_open: bool,
) -> Result<(), Error> {
    let _ = sqlx::query!(
        r#"
        UPDATE "group"
        SET is_open=?1
        WHERE id=?2
        "#,
        is_open,
        group_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn update_group_owner(
    connection: &mut SqliteConnection,
    group_id: &GroupId,
    owner_id: &UserId,
) -> Result<(), Error> {
    let _ = sqlx::query!(
        r#"
        UPDATE "group"
        SET owner_id=?1
        WHERE id=?2
        "#,
        owner_id,
        group_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn delete_group(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    group_id: &GroupId,
) -> Result<bool, Error> {
    let _ = sqlx::query!(
        r#"
        DELETE FROM group_association
        WHERE group_id=?1
        "#,
        group_id
    )
    .execute(&mut *connection)
    .await?;

    let result = sqlx::query!(
        r#"
        UPDATE "group"
        SET
        deleted_at=?2
        WHERE id=?1
        "#,
        group_id,
        now,
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_group_members(
    connection: &mut SqliteConnection,
    group_id: &GroupId,
) -> Result<Vec<User>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT
            u.id AS "u_id: UserId",
            u.provider AS "u_provider: IdentityProvider",
            u.foreign_id AS "u_foreign_id: String",
            u.name AS "u_name: String",
            u.created_at AS "u_created_at: DateTime<Utc>",
            u.deactivated_at AS "u_deactivated_at: DateTime<Utc>",
            u.license_plate_number AS "u_license_plate_number: String"
        FROM
            group_association "ga"
        JOIN
            user "u" ON ga.user_id = u.id
        WHERE
            ga.group_id = ?1
        "#,
        group_id
    )
    .fetch_all(&mut *connection)
    .await?;

    Ok(result
        .into_iter()
        .map(|v| User {
            id: v.u_id,
            provider: v.u_provider,
            foreign_id: v.u_foreign_id,
            name: v.u_name,
            created_at: v.u_created_at,
            deactivated_at: v.u_deactivated_at,
            license_plate_number: v.u_license_plate_number,
        })
        .collect())
}

pub async fn is_member_of(
    connection: &mut SqliteConnection,
    group_id: &GroupId,
    user_id: &UserId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(*) AS "count"
        FROM group_association
        WHERE group_id=?1 AND user_id=?2
        "#,
        group_id,
        user_id,
    )
    .fetch_one(&mut *connection)
    .await?;

    Ok(result.count > 0)
}

pub async fn join_group(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    group_id: &GroupId,
    user_id: &UserId,
) -> Result<(), Error> {
    let _ = sqlx::query!(
        r#"
        INSERT INTO group_association(group_id, user_id, joined_at)
        VALUES(?1, ?2, ?3)
        "#r,
        group_id,
        user_id,
        now
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn leave_group(
    connection: &mut SqliteConnection,
    group_id: &GroupId,
    user_id: &UserId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM group_association
        WHERE group_id = ?1 AND user_id = ?2
        "#r,
        group_id,
        user_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}
