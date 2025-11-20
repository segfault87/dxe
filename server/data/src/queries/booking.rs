use chrono::{DateTime, Utc};
use sqlx::SqliteConnection;

use crate::Error;
use crate::entities::{
    Booking, CashPaymentStatus, Group, Identity, IdentityDiscriminator, ItsokeyCredential,
    OccupiedSlot, Reservation, Unit, User,
};
use crate::types::{
    BookingId, GroupId, IdentityId, IdentityProvider, ReservationId, UnitId, UserId,
};

pub async fn is_unit_enabled(
    connection: &mut SqliteConnection,
    unit_id: &UnitId,
) -> Result<Option<bool>, Error> {
    let result = sqlx::query_as!(
        Unit,
        r#"
        SELECT
            id as "id: _",
            enabled
        FROM unit
        WHERE id = ?1
        "#,
        unit_id
    )
    .fetch_optional(&mut *connection)
    .await?;

    Ok(result.map(|v| v.enabled))
}

pub async fn is_booking_available(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    unit_id: &UnitId,
    time_from: &DateTime<Utc>,
    time_to: &DateTime<Utc>,
) -> Result<bool, Error> {
    Ok(sqlx::query!(
        r#"
            SELECT
                (SELECT COUNT(*)
                FROM booking
                WHERE
                    time_to >= ?1 AND
                    MAX(time_from, ?1) < MIN(time_to, ?2) AND
                    (canceled_at IS NULL OR canceled_at > ?4) AND
                    unit_id = ?3) +
                (SELECT COUNT(*)
                FROM reservation
                WHERE
                    time_to >= ?1 AND
                    MAX(time_from, ?1) < MIN(time_to, ?2) AND
                    unit_id = ?3)
                AS "count"
        "#,
        time_from,
        time_to,
        unit_id,
        now
    )
    .fetch_one(&mut *connection)
    .await?
    .count
        == 0)
}

pub async fn get_occupied_slots(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    unit_id: &UnitId,
    time_from: &DateTime<Utc>,
    time_to: &DateTime<Utc>,
) -> Result<Vec<OccupiedSlot>, Error> {
    let mut records = vec![];

    let bookings = sqlx::query!(
        r#"
        SELECT
            b.time_from AS "time_from: DateTime<Utc>",
            b.time_to AS "time_to: DateTime<Utc>",
            cu.name OR cg.name AS "name: String",
            b.confirmed_at AS "confirmed_at: DateTime<Utc>"
        FROM booking "b"
        JOIN identity "ci" ON b.customer_id = ci.id
        LEFT OUTER JOIN user "cu" ON ci.discriminator = 'user' AND ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON ci.discriminator = 'group' AND ci.id = cg.id
        WHERE
            b.time_from >= ?1 AND b.time_from < ?2 AND
            b.unit_id = ?3 AND
            (b.canceled_at IS NULL OR b.canceled_at >= ?4)
        "#,
        time_from,
        time_to,
        unit_id,
        now
    )
    .fetch_all(&mut *connection)
    .await?;

    for booking in bookings {
        records.push(OccupiedSlot {
            name: booking.name.unwrap_or_default(),
            time_from: booking.time_from,
            time_to: booking.time_to,
            confirmed: booking.confirmed_at.map(|v| &v < now).unwrap_or(false),
        })
    }

    let reservations = sqlx::query!(
        r#"
        SELECT
            remark,
            time_from AS "time_from: DateTime<Utc>",
            time_to AS "time_to: DateTime<Utc>"
        FROM reservation
        WHERE
            time_from >= ?1 AND time_from < ?2 AND
            unit_id = ?3
        "#,
        time_from,
        time_to,
        unit_id
    )
    .fetch_all(&mut *connection)
    .await?;

    for reservation in reservations {
        records.push(OccupiedSlot {
            name: reservation.remark.unwrap_or_default(),
            time_from: reservation.time_from,
            time_to: reservation.time_to,
            confirmed: true,
        });
    }

    Ok(records)
}

pub async fn get_booking(
    connection: &mut SqliteConnection,
    booking_id: &BookingId,
) -> Result<Option<Booking>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT
            b.id AS "b_id: BookingId",
            b.unit_id AS "b_unit_id: UnitId",
            hu.id AS "hu_id: UserId",
            hu.provider AS "hu_provider: IdentityProvider",
            hu.foreign_id AS "hu_foreign_id",
            hu.name AS "hu_name",
            hu.created_at AS "hu_created_at: DateTime<Utc>",
            hu.deactivated_at AS "hu_deactivated_at: DateTime<Utc>",
            hu.license_plate_number AS "hu_license_plate_number",
            ci.discriminator AS "ci_discriminator: IdentityDiscriminator",
            cu.id AS "cu_id: Option<UserId>",
            cu.provider AS "cu_provider: Option<IdentityProvider>",
            cu.foreign_id AS "cu_foreign_id: Option<String>",
            cu.name AS "cu_name: Option<String>",
            cu.created_at AS "cu_created_at: Option<DateTime<Utc>>",
            cu.deactivated_at AS "cu_deactivated_at: DateTime<Utc>",
            cu.license_plate_number AS "cu_license_plate_number",
            cg.id AS "cg_id: Option<GroupId>",
            cg.name AS "cg_name: Option<String>",
            cg.owner_id AS "cg_owner_id: Option<UserId>",
            cg.is_open AS "cg_is_open: Option<bool>",
            cg.created_at AS "cg_created_at: Option<DateTime<Utc>>",
            cg.deleted_at AS "cg_deleted_at: DateTime<Utc>",
            b.time_from AS "b_time_from: DateTime<Utc>",
            b.time_to AS "b_time_to: DateTime<Utc>",
            b.created_at AS "b_created_at: DateTime<Utc>",
            b.confirmed_at AS "b_confirmed_at: DateTime<Utc>",
            b.canceled_at AS "b_canceled_at: DateTime<Utc>"
        FROM booking "b"
        JOIN user "hu" ON b.holder_id = hu.id
        JOIN identity "ci" ON b.customer_id = ci.id
        LEFT OUTER JOIN user "cu" ON
            ci.discriminator = 'user' AND
            ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON
            ci.discriminator = 'group' AND
            ci.id = cg.id
        WHERE
            b.id = ?1
        "#,
        booking_id,
    )
    .fetch_optional(&mut *connection)
    .await?;

    if let Some(v) = result {
        Ok(Some(Booking {
            id: v.b_id,
            unit_id: v.b_unit_id,
            holder: User {
                id: v.hu_id,
                provider: v.hu_provider,
                foreign_id: v.hu_foreign_id,
                name: v.hu_name,
                created_at: v.hu_created_at,
                deactivated_at: v.hu_deactivated_at,
                license_plate_number: v.hu_license_plate_number,
            },
            customer: match v.ci_discriminator {
                IdentityDiscriminator::User => Identity::User(User {
                    id: v.cu_id.ok_or(Error::MissingField("cu_id"))?,
                    provider: v.cu_provider.ok_or(Error::MissingField("cu_provider"))?,
                    foreign_id: v
                        .cu_foreign_id
                        .ok_or(Error::MissingField("cu_foreign_id"))?,
                    name: v.cu_name.ok_or(Error::MissingField("cu_name"))?,
                    created_at: v
                        .cu_created_at
                        .ok_or(Error::MissingField("cu_created_at"))?,
                    deactivated_at: v.cu_deactivated_at,
                    license_plate_number: v.cu_license_plate_number,
                }),
                IdentityDiscriminator::Group => Identity::Group(Group {
                    id: v.cg_id.ok_or(Error::MissingField("cg_id"))?,
                    name: v.cg_name.ok_or(Error::MissingField("cg_name"))?,
                    owner_id: v.cg_owner_id.ok_or(Error::MissingField("cg_owner_id"))?,
                    is_open: v.cg_is_open.ok_or(Error::MissingField("cg_is_open"))?,
                    created_at: v
                        .cg_created_at
                        .ok_or(Error::MissingField("cg_created_at"))?,
                    deleted_at: v.cg_deleted_at,
                }),
            },
            time_from: v.b_time_from,
            time_to: v.b_time_to,
            created_at: v.b_created_at,
            confirmed_at: v.b_confirmed_at,
            canceled_at: v.b_canceled_at,
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_booking_with_user_id(
    connection: &mut SqliteConnection,
    booking_id: &BookingId,
    user_id: &UserId,
) -> Result<Option<Booking>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT
            b.id AS "b_id: BookingId",
            b.unit_id AS "b_unit_id: UnitId",
            hu.id AS "hu_id: UserId",
            hu.provider AS "hu_provider: IdentityProvider",
            hu.foreign_id AS "hu_foreign_id",
            hu.name AS "hu_name",
            hu.created_at AS "hu_created_at: DateTime<Utc>",
            hu.deactivated_at AS "hu_deactivated_at: DateTime<Utc>",
            hu.license_plate_number AS "hu_license_plate_number",
            ci.discriminator AS "ci_discriminator: IdentityDiscriminator",
            cu.id AS "cu_id: Option<UserId>",
            cu.provider AS "cu_provider: Option<IdentityProvider>",
            cu.foreign_id AS "cu_foreign_id: Option<String>",
            cu.name AS "cu_name: Option<String>",
            cu.created_at AS "cu_created_at: Option<DateTime<Utc>>",
            cu.deactivated_at AS "cu_deactivated_at: DateTime<Utc>",
            cu.license_plate_number AS "cu_license_plate_number",
            cg.id AS "cg_id: Option<GroupId>",
            cg.name AS "cg_name: Option<String>",
            cg.owner_id AS "cg_owner_id: Option<UserId>",
            cg.is_open AS "cg_is_open: Option<bool>",
            cg.created_at AS "cg_created_at: Option<DateTime<Utc>>",
            cg.deleted_at AS "cg_deleted_at: DateTime<Utc>",
            b.time_from AS "b_time_from: DateTime<Utc>",
            b.time_to AS "b_time_to: DateTime<Utc>",
            b.created_at AS "b_created_at: DateTime<Utc>",
            b.confirmed_at AS "b_confirmed_at: DateTime<Utc>",
            b.canceled_at AS "b_canceled_at: DateTime<Utc>"
        FROM booking "b"
        JOIN user "hu" ON b.holder_id = hu.id
        JOIN identity "ci" ON b.customer_id = ci.id
        LEFT OUTER JOIN user "cu" ON
            ci.discriminator = 'user' AND
            ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON
            ci.discriminator = 'group' AND
            ci.id = cg.id
        WHERE
            b.id = ?1 AND
            (cu.id = ?2 OR EXISTS (SELECT user_id FROM group_association "ga" WHERE ga.group_id = cg.id AND ga.user_id = ?2))
        "#,
        booking_id,
        user_id
    )
    .fetch_optional(&mut *connection)
    .await?;

    if let Some(v) = result {
        Ok(Some(Booking {
            id: v.b_id,
            unit_id: v.b_unit_id,
            holder: User {
                id: v.hu_id,
                provider: v.hu_provider,
                foreign_id: v.hu_foreign_id,
                name: v.hu_name,
                created_at: v.hu_created_at,
                deactivated_at: v.hu_deactivated_at,
                license_plate_number: v.hu_license_plate_number,
            },
            customer: match v.ci_discriminator {
                IdentityDiscriminator::User => Identity::User(User {
                    id: v.cu_id.ok_or(Error::MissingField("cu_id"))?,
                    provider: v.cu_provider.ok_or(Error::MissingField("cu_provider"))?,
                    foreign_id: v
                        .cu_foreign_id
                        .ok_or(Error::MissingField("cu_foreign_id"))?,
                    name: v.cu_name.ok_or(Error::MissingField("cu_name"))?,
                    created_at: v
                        .cu_created_at
                        .ok_or(Error::MissingField("cu_created_at"))?,
                    deactivated_at: v.cu_deactivated_at,
                    license_plate_number: v.cu_license_plate_number,
                }),
                IdentityDiscriminator::Group => Identity::Group(Group {
                    id: v.cg_id.ok_or(Error::MissingField("cg_id"))?,
                    name: v.cg_name.ok_or(Error::MissingField("cg_name"))?,
                    owner_id: v.cg_owner_id.ok_or(Error::MissingField("cg_owner_id"))?,
                    is_open: v.cg_is_open.ok_or(Error::MissingField("cg_is_open"))?,
                    created_at: v
                        .cg_created_at
                        .ok_or(Error::MissingField("cg_created_at"))?,
                    deleted_at: v.cg_deleted_at,
                }),
            },
            time_from: v.b_time_from,
            time_to: v.b_time_to,
            created_at: v.b_created_at,
            confirmed_at: v.b_confirmed_at,
            canceled_at: v.b_canceled_at,
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_bookings_by_unit_id(
    connection: &mut SqliteConnection,
    unit_id: &UnitId,
    time_from: &DateTime<Utc>,
    time_to: &DateTime<Utc>,
) -> Result<Vec<Booking>, Error> {
    sqlx::query!(
        r#"
        SELECT
            b.id AS "b_id: BookingId",
            b.unit_id AS "b_unit_id: UnitId",
            hu.id AS "hu_id: UserId",
            hu.provider AS "hu_provider: IdentityProvider",
            hu.foreign_id AS "hu_foreign_id",
            hu.name AS "hu_name",
            hu.created_at AS "hu_created_at: DateTime<Utc>",
            hu.deactivated_at AS "hu_deactivated_at: DateTime<Utc>",
            hu.license_plate_number AS "hu_license_plate_number",
            ci.discriminator AS "ci_discriminator: IdentityDiscriminator",
            cu.id AS "cu_id: Option<UserId>",
            cu.provider AS "cu_provider: Option<IdentityProvider>",
            cu.foreign_id AS "cu_foreign_id: Option<String>",
            cu.name AS "cu_name: Option<String>",
            cu.created_at AS "cu_created_at: Option<DateTime<Utc>>",
            cu.deactivated_at AS "cu_deactivated_at: DateTime<Utc>",
            cu.license_plate_number AS "cu_license_plate_number",
            cg.id AS "cg_id: Option<GroupId>",
            cg.name AS "cg_name: Option<String>",
            cg.owner_id AS "cg_owner_id: Option<UserId>",
            cg.is_open AS "cg_is_open: Option<bool>",
            cg.created_at AS "cg_created_at: Option<DateTime<Utc>>",
            cg.deleted_at AS "cg_deleted_at: DateTime<Utc>",
            b.time_from AS "b_time_from: DateTime<Utc>",
            b.time_to AS "b_time_to: DateTime<Utc>",
            b.created_at AS "b_created_at: DateTime<Utc>",
            b.confirmed_at AS "b_confirmed_at: DateTime<Utc>",
            b.canceled_at AS "b_canceled_at: DateTime<Utc>"
        FROM booking "b"
        JOIN user "hu" ON b.holder_id = hu.id
        JOIN identity "ci" ON b.customer_id = ci.id
        LEFT OUTER JOIN user "cu" ON ci.discriminator = 'user' AND ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON ci.discriminator = 'group' AND ci.id = cg.id
        WHERE
            b.unit_id = ?1 AND
            b.time_from >= ?2 AND
            b.time_to < ?3 AND
            b.canceled_at IS NULL
        "#,
        unit_id,
        time_from,
        time_to,
    )
    .fetch_optional(&mut *connection)
    .await?
    .into_iter()
    .map(|v| {
        Ok(Booking {
            id: v.b_id,
            unit_id: v.b_unit_id,
            holder: User {
                id: v.hu_id,
                provider: v.hu_provider,
                foreign_id: v.hu_foreign_id,
                name: v.hu_name,
                created_at: v.hu_created_at,
                deactivated_at: v.hu_deactivated_at,
                license_plate_number: v.hu_license_plate_number,
            },
            customer: match v.ci_discriminator {
                IdentityDiscriminator::User => Identity::User(User {
                    id: v.cu_id.ok_or(Error::MissingField("cu_id"))?,
                    provider: v.cu_provider.ok_or(Error::MissingField("cu_provider"))?,
                    foreign_id: v
                        .cu_foreign_id
                        .ok_or(Error::MissingField("cu_foreign_id"))?,
                    name: v.cu_name.ok_or(Error::MissingField("cu_name"))?,
                    created_at: v
                        .cu_created_at
                        .ok_or(Error::MissingField("cu_created_at"))?,
                    deactivated_at: v.cu_deactivated_at,
                    license_plate_number: v.cu_license_plate_number,
                }),
                IdentityDiscriminator::Group => Identity::Group(Group {
                    id: v.cg_id.ok_or(Error::MissingField("cg_id"))?,
                    name: v.cg_name.ok_or(Error::MissingField("cg_name"))?,
                    owner_id: v.cg_owner_id.ok_or(Error::MissingField("cg_owner_id"))?,
                    is_open: v.cg_is_open.ok_or(Error::MissingField("cg_is_open"))?,
                    created_at: v
                        .cg_created_at
                        .ok_or(Error::MissingField("cg_created_at"))?,
                    deleted_at: v.cg_deleted_at,
                }),
            },
            time_from: v.b_time_from,
            time_to: v.b_time_to,
            created_at: v.b_created_at,
            confirmed_at: v.b_confirmed_at,
            canceled_at: v.b_canceled_at,
        })
    })
    .collect::<Result<Vec<_>, Error>>()
}

pub async fn get_bookings_by_user_id(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    user_id: &UserId,
    time_from: &DateTime<Utc>,
    include_canceled: bool,
) -> Result<Vec<Booking>, Error> {
    sqlx::query!(
        r#"
        SELECT
            b.id AS "b_id: BookingId",
            b.unit_id AS "b_unit_id: UnitId",
            hu.id AS "hu_id: UserId",
            hu.provider AS "hu_provider: IdentityProvider",
            hu.foreign_id AS "hu_foreign_id",
            hu.name AS "hu_name",
            hu.created_at AS "hu_created_at: DateTime<Utc>",
            hu.deactivated_at AS "hu_deactivated_at: DateTime<Utc>",
            hu.license_plate_number AS "hu_license_plate_number",
            ci.id AS "ci_id: IdentityId",
            ci.discriminator AS "ci_discriminator: IdentityDiscriminator",
            cu.id AS "cu_id: Option<UserId>",
            cu.provider AS "cu_provider: Option<IdentityProvider>",
            cu.foreign_id AS "cu_foreign_id: Option<String>",
            cu.name AS "cu_name: Option<String>",
            cu.created_at AS "cu_created_at: Option<DateTime<Utc>>",
            cu.deactivated_at AS "cu_deactivated_at: DateTime<Utc>",
            cu.license_plate_number AS "cu_license_plate_number",
            cg.id AS "cg_id: Option<GroupId>",
            cg.name AS "cg_name: Option<String>",
            cg.owner_id AS "cg_owner_id: Option<UserId>",
            cg.is_open AS "cg_is_open: Option<bool>",
            cg.created_at AS "cg_created_at: Option<DateTime<Utc>>",
            cg.deleted_at AS "cg_deleted_at: DateTime<Utc>",
            b.time_from AS "b_time_from: DateTime<Utc>",
            b.time_to AS "b_time_to: DateTime<Utc>",
            b.created_at AS "b_created_at: DateTime<Utc>",
            b.confirmed_at AS "b_confirmed_at: DateTime<Utc>",
            b.canceled_at AS "b_canceled_at: DateTime<Utc>"
        FROM booking "b"
        JOIN user "hu" ON b.holder_id = hu.id
        JOIN identity "ci" ON b.customer_id = ci.id
        LEFT OUTER JOIN user "cu" ON ci.discriminator = 'user' AND ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON ci.discriminator = 'group' AND ci.id = cg.id
        WHERE
            b.time_to >= ?2 AND
            (
                cu.id = ?1 OR
                EXISTS (
                    SELECT user_id FROM group_association "ga" WHERE ga.group_id = cg.id AND ga.user_id = ?1
                )
            )
        "#,
        user_id,
        time_from,
    )
    .fetch_all(&mut *connection)
    .await?
    .into_iter()
    .filter(|v| {
        if !include_canceled && let Some(canceled_at) = v.b_canceled_at.as_ref() && canceled_at < now {
            false
        } else {
            true
        }
    })
    .map(|v| {
        Ok(Booking {
            id: v.b_id,
            unit_id: v.b_unit_id,
            holder: User {
                id: v.hu_id,
                provider: v.hu_provider,
                foreign_id: v.hu_foreign_id,
                name: v.hu_name,
                created_at: v.hu_created_at,
                deactivated_at: v.hu_deactivated_at,
                license_plate_number: v.hu_license_plate_number,
            },
            customer: match v.ci_discriminator {
                IdentityDiscriminator::User => Identity::User(User {
                    id: v.cu_id.ok_or(Error::MissingField("cu_id"))?,
                    provider: v.cu_provider.ok_or(Error::MissingField("cu_provider"))?,
                    foreign_id: v
                        .cu_foreign_id
                        .ok_or(Error::MissingField("cu_foreign_id"))?,
                    name: v.cu_name.ok_or(Error::MissingField("cu_name"))?,
                    created_at: v
                        .cu_created_at
                        .ok_or(Error::MissingField("cu_created_at"))?,
                    deactivated_at: v.cu_deactivated_at,
                    license_plate_number: v.cu_license_plate_number,
                }),
                IdentityDiscriminator::Group => Identity::Group(Group {
                    id: v.cg_id.ok_or(Error::MissingField("cg_id"))?,
                    name: v.cg_name.ok_or(Error::MissingField("cg_name"))?,
                    owner_id: v
                        .cg_owner_id
                        .ok_or(Error::MissingField("cg_owner_id"))?,
                    is_open: v.cg_is_open.ok_or(Error::MissingField("cg_is_open"))?,
                    created_at: v
                        .cg_created_at
                        .ok_or(Error::MissingField("cg_created_at"))?,
                    deleted_at: v.cg_deleted_at,
                }),
            },
            time_from: v.b_time_from,
            time_to: v.b_time_to,
            created_at: v.b_created_at,
            confirmed_at: v.b_confirmed_at,
            canceled_at: v.b_canceled_at,
        })
    })
    .collect::<Result<Vec<_>, Error>>()
}

pub async fn get_bookings_pending(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    include_canceled: bool,
) -> Result<Vec<(Booking, Option<CashPaymentStatus>)>, Error> {
    sqlx::query!(
        r#"
        SELECT
            b.id AS "b_id: BookingId",
            b.unit_id AS "b_unit_id: UnitId",
            hu.id AS "hu_id: UserId",
            hu.provider AS "hu_provider: IdentityProvider",
            hu.foreign_id AS "hu_foreign_id",
            hu.name AS "hu_name",
            hu.created_at AS "hu_created_at: DateTime<Utc>",
            hu.deactivated_at AS "hu_deactivated_at: DateTime<Utc>",
            hu.license_plate_number AS "hu_license_plate_number",
            ci.id AS "ci_id: IdentityId",
            ci.discriminator AS "ci_discriminator: IdentityDiscriminator",
            cu.id AS "cu_id: UserId",
            cu.provider AS "cu_provider: IdentityProvider",
            cu.foreign_id AS "cu_foreign_id: String",
            cu.name AS "cu_name: String",
            cu.created_at AS "cu_created_at: DateTime<Utc>",
            cu.deactivated_at AS "cu_deactivated_at: DateTime<Utc>",
            cu.license_plate_number AS "cu_license_plate_number",
            cg.id AS "cg_id: GroupId",
            cg.name AS "cg_name: String",
            cg.owner_id AS "cg_owner_id: UserId",
            cg.is_open AS "cg_is_open: bool",
            cg.created_at AS "cg_created_at: DateTime<Utc>",
            cg.deleted_at AS "cg_deleted_at: DateTime<Utc>",
            b.time_from AS "b_time_from: DateTime<Utc>",
            b.time_to AS "b_time_to: DateTime<Utc>",
            b.created_at AS "b_created_at: DateTime<Utc>",
            b.confirmed_at AS "b_confirmed_at: DateTime<Utc>",
            b.canceled_at AS "b_canceled_at: DateTime<Utc>",
            cps.depositor_name AS "cps_depsitor_name: String",
            cps.price AS "cps_price: i64",
            cps.created_at AS "cps_created_at: DateTime<Utc>",
            cps.confirmed_at AS "cps_confirmed_at: DateTime<Utc>",
            cps.refund_account AS "cps_refund_account: String",
            cps.refund_price AS "cps_refund_price: i64",
            cps.refunded_at AS "cps_refunded_at: DateTime<Utc>"
        FROM booking "b"
        JOIN user "hu" ON b.holder_id = hu.id
        JOIN identity "ci" ON b.customer_id = ci.id
        LEFT OUTER JOIN user "cu" ON ci.discriminator = 'user' AND ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON ci.discriminator = 'group' AND ci.id = cg.id
        LEFT OUTER JOIN cash_payment_status "cps" ON b.id = cps.booking_id
        WHERE
            b.confirmed_at IS NULL OR b.confirmed_at >= ?1
        ORDER BY b.created_at DESC
        "#,
        now,
    )
    .fetch_all(&mut *connection)
    .await?
    .into_iter()
    .filter(|v| {
        if !include_canceled
            && let Some(canceled_at) = v.b_canceled_at.as_ref()
            && canceled_at < now
        {
            false
        } else {
            true
        }
    })
    .map(|v| {
        let cash_payment_status = if let Some(cps_created_at) = v.cps_created_at {
            Some(CashPaymentStatus {
                booking_id: v.b_id,
                created_at: cps_created_at,
                depositor_name: v
                    .cps_depsitor_name
                    .ok_or(Error::MissingField("cps_depositor_name"))?,
                price: v.cps_price.ok_or(Error::MissingField("cps_price"))?,
                confirmed_at: v.cps_confirmed_at,
                refund_account: v.cps_refund_account,
                refund_price: v.cps_refund_price,
                refunded_at: v.cps_refunded_at,
            })
        } else {
            None
        };

        Ok((
            Booking {
                id: v.b_id,
                unit_id: v.b_unit_id,
                holder: User {
                    id: v.hu_id,
                    provider: v.hu_provider,
                    foreign_id: v.hu_foreign_id,
                    name: v.hu_name,
                    created_at: v.hu_created_at,
                    deactivated_at: v.hu_deactivated_at,
                    license_plate_number: v.hu_license_plate_number,
                },
                customer: match v.ci_discriminator {
                    IdentityDiscriminator::User => Identity::User(User {
                        id: v.cu_id.ok_or(Error::MissingField("cu_id"))?,
                        provider: v.cu_provider.ok_or(Error::MissingField("cu_provider"))?,
                        foreign_id: v
                            .cu_foreign_id
                            .ok_or(Error::MissingField("cu_foreign_id"))?,
                        name: v.cu_name.ok_or(Error::MissingField("cu_name"))?,
                        created_at: v
                            .cu_created_at
                            .ok_or(Error::MissingField("cu_created_at"))?,
                        deactivated_at: v.cu_deactivated_at,
                        license_plate_number: v.cu_license_plate_number,
                    }),
                    IdentityDiscriminator::Group => Identity::Group(Group {
                        id: v.cg_id.ok_or(Error::MissingField("cg_id"))?,
                        name: v.cg_name.ok_or(Error::MissingField("cg_name"))?,
                        owner_id: v.cg_owner_id.ok_or(Error::MissingField("cg_owner_id"))?,
                        is_open: v.cg_is_open.ok_or(Error::MissingField("cg_is_open"))?,
                        created_at: v
                            .cg_created_at
                            .ok_or(Error::MissingField("cg_created_at"))?,
                        deleted_at: v.cg_deleted_at,
                    }),
                },
                time_from: v.b_time_from,
                time_to: v.b_time_to,
                created_at: v.b_created_at,
                confirmed_at: v.b_confirmed_at,
                canceled_at: v.b_canceled_at,
            },
            cash_payment_status,
        ))
    })
    .collect::<Result<Vec<_>, Error>>()
}

pub async fn create_booking(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    unit_id: &UnitId,
    user_id: &UserId,
    customer_id: &IdentityId,
    time_from: &DateTime<Utc>,
    time_to: &DateTime<Utc>,
) -> Result<BookingId, Error> {
    if is_unit_enabled(connection, unit_id).await? != Some(true) {
        return Err(Error::UnitNotFound);
    }

    if !is_booking_available(connection, now, unit_id, time_from, time_to).await? {
        return Err(Error::TimeRangeOccupied);
    }

    let booking_id = BookingId::generate();

    sqlx::query!(
        r#"
        INSERT INTO booking(id, unit_id, holder_id, customer_id, time_from, time_to, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        booking_id,
        unit_id,
        user_id,
        customer_id,
        time_from,
        time_to,
        now
    )
    .execute(&mut *connection)
    .await?;

    Ok(booking_id)
}

pub async fn update_booking_customer(
    connection: &mut SqliteConnection,
    booking_id: &BookingId,
    new_identity_id: &IdentityId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE booking
        SET customer_id=?1
        WHERE id=?2
        "#,
        new_identity_id,
        booking_id,
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn create_cash_payment_status(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    booking_id: &BookingId,
    depositor_name: &str,
    price: i64,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO cash_payment_status(booking_id, depositor_name, price, created_at)
        VALUES(?1, ?2, ?3, ?4)
        "#,
        booking_id,
        depositor_name,
        price,
        now,
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn get_cash_payment_status(
    connection: &mut SqliteConnection,
    booking_id: &BookingId,
) -> Result<Option<CashPaymentStatus>, Error> {
    Ok(sqlx::query_as!(
        CashPaymentStatus,
        r#"
        SELECT
            booking_id AS "booking_id: _",
            depositor_name AS "depositor_name: _",
            price AS "price: _",
            created_at AS "created_at: _",
            confirmed_at AS "confirmed_at: _",
            refund_price AS "refund_price: _",
            refund_account AS "refund_account: _",
            refunded_at AS "refunded_at: _"
        FROM cash_payment_status
        WHERE booking_id=?1
        "#,
        booking_id
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn confirm_cash_payment(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    booking_id: &BookingId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE cash_payment_status
        SET confirmed_at=?1
        WHERE booking_id=?2 AND confirmed_at IS NULL
        "#,
        now,
        booking_id,
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn update_refund_information(
    connection: &mut SqliteConnection,
    booking_id: &BookingId,
    refund_price: i64,
    refund_account: Option<String>,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE cash_payment_status
        SET refund_price=?1, refund_account=?2
        WHERE booking_id=?3 AND refunded_at IS NULL
        "#,
        refund_price,
        refund_account,
        booking_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn refund_payment(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    booking_id: &BookingId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE cash_payment_status
        SET refunded_at=?1
        WHERE booking_id=?2 AND refunded_at IS NULL
        "#,
        now,
        booking_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn confirm_booking(
    connection: &mut SqliteConnection,
    booking_id: &BookingId,
    now: &DateTime<Utc>,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE booking
        SET confirmed_at = ?2
        WHERE id = ?1 AND confirmed_at IS NULL
        "#,
        booking_id,
        now
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn cancel_booking(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    booking_id: &BookingId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE booking
        SET canceled_at = ?2
        WHERE id = ?1 AND canceled_at IS NULL
        "#,
        booking_id,
        now
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn create_reservation(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    unit_id: &UnitId,
    user_id: &UserId,
    time_from: &DateTime<Utc>,
    time_to: &DateTime<Utc>,
    remark: &Option<String>,
    temporary: bool,
) -> Result<ReservationId, Error> {
    if is_unit_enabled(connection, unit_id).await? != Some(true) {
        return Err(Error::UnitNotFound);
    }

    if !is_booking_available(connection, now, unit_id, time_from, time_to).await? {
        return Err(Error::TimeRangeOccupied);
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO reservation(unit_id, holder_id, time_from, time_to, remark, temporary)
        VALUES(?1, ?2, ?3, ?4, ?5, ?6)
        RETURNING id
        "#,
        unit_id,
        user_id,
        time_from,
        time_to,
        remark,
        temporary
    )
    .fetch_one(&mut *connection)
    .await?;

    Ok(result.id.into())
}

pub async fn get_reservation(
    connection: &mut SqliteConnection,
    id: ReservationId,
) -> Result<Option<Reservation>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT
            r.id AS "id: ReservationId",
            r.unit_id AS "unit_id: UnitId",
            r.time_from AS "time_from: DateTime<Utc>",
            r.time_to AS "time_to: DateTime<Utc>",
            r.temporary,
            r.remark,
            u.id AS "u_id: UserId",
            u.provider AS "u_provider: IdentityProvider",
            u.foreign_id AS "u_foreign_id: String",
            u.name AS "u_name: String",
            u.created_at AS "u_created_at: DateTime<Utc>",
            u.deactivated_at AS "u_deactivated_at: DateTime<Utc>",
            u.license_plate_number AS "u_license_plate_number"
        FROM reservation "r"
        JOIN user "u" ON r.holder_id=u.id
        WHERE
            r.id = ?1
        "#,
        id
    )
    .fetch_optional(&mut *connection)
    .await?;

    Ok(result.map(|v| Reservation {
        id: v.id,
        unit_id: v.unit_id,
        time_from: v.time_from,
        time_to: v.time_to,
        temporary: v.temporary,
        remark: v.remark,
        holder: User {
            id: v.u_id,
            provider: v.u_provider,
            foreign_id: v.u_foreign_id,
            name: v.u_name,
            created_at: v.u_created_at,
            deactivated_at: v.u_deactivated_at,
            license_plate_number: v.u_license_plate_number,
        },
    }))
}

pub async fn delete_reservation(
    connection: &mut SqliteConnection,
    reservation_id: ReservationId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM reservation WHERE id=?1
        "#,
        reservation_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_reservations_by_unit_id(
    connection: &mut SqliteConnection,
    unit_id: &UnitId,
    date_from: Option<DateTime<Utc>>,
) -> Result<Vec<Reservation>, Error> {
    let date_from = date_from.unwrap_or_default();

    let result = sqlx::query!(
        r#"
        SELECT
            r.id AS "id: ReservationId",
            r.unit_id AS "unit_id: UnitId",
            r.time_from AS "time_from: DateTime<Utc>",
            r.time_to AS "time_to: DateTime<Utc>",
            r.temporary,
            r.remark,
            u.id AS "u_id: UserId",
            u.provider AS "u_provider: IdentityProvider",
            u.foreign_id AS "u_foreign_id: String",
            u.name AS "u_name: String",
            u.created_at AS "u_created_at: DateTime<Utc>",
            u.deactivated_at AS "u_deactivated_at: DateTime<Utc>",
            u.license_plate_number AS "u_license_plate_number"
        FROM reservation "r"
        JOIN user "u" ON r.holder_id=u.id
        WHERE
            unit_id = ?1 AND
            time_to > ?2
        ORDER BY r.time_from ASC
        "#,
        unit_id,
        date_from
    )
    .fetch_all(&mut *connection)
    .await?;

    Ok(result
        .into_iter()
        .map(|v| Reservation {
            id: v.id,
            unit_id: v.unit_id,
            time_from: v.time_from,
            time_to: v.time_to,
            temporary: v.temporary,
            remark: v.remark,
            holder: User {
                id: v.u_id,
                provider: v.u_provider,
                foreign_id: v.u_foreign_id,
                name: v.u_name,
                created_at: v.u_created_at,
                deactivated_at: v.u_deactivated_at,
                license_plate_number: v.u_license_plate_number,
            },
        })
        .collect())
}

pub async fn create_itsokey_credential(
    connection: &mut SqliteConnection,
    booking_id: &BookingId,
    key: &str,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO itsokey_credential(booking_id, key)
        VALUES(?1, ?2)
        "#,
        booking_id,
        key
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn get_itsokey_credential(
    connection: &mut SqliteConnection,
    booking_id: &BookingId,
) -> Result<Option<ItsokeyCredential>, Error> {
    Ok(sqlx::query_as!(
        ItsokeyCredential,
        r#"
        SELECT
            booking_id AS "booking_id: _",
            key
        FROM itsokey_credential
        WHERE booking_id=?1
        "#,
        booking_id
    )
    .fetch_optional(&mut *connection)
    .await?)
}
