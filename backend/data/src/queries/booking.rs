use chrono::{DateTime, Utc};
use dxe_types::{
    AdhocParkingId, AdhocReservationId, BookingAmendmentId, BookingId, GroupId, IdentityId,
    IdentityProvider, ProductId, SpaceId, TelemetryType, UnitId, UserId,
};
use sqlx::SqliteConnection;

use crate::Error;
use crate::entities::{
    AdhocParking, AdhocReservation, AudioRecording, Booking, BookingAmendment, CashTransaction,
    Group, Identity, IdentityDiscriminator, OccupiedSlot, Product, ProductDiscriminator,
    TelemetryFile, User,
};
use crate::queries::unit::is_unit_enabled;
use crate::utils::is_in_effect;

const MAX_ARBITRARY_DATETIME_RANGE: DateTime<Utc> = DateTime::from_timestamp_nanos(i64::MAX);

pub async fn is_booking_available(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    unit_id: &UnitId,
    range_from: &DateTime<Utc>,
    range_to: &DateTime<Utc>,
    exclude_booking_id: Option<&BookingId>,
    exclude_adhoc_reservation_id: Option<&AdhocReservationId>,
) -> Result<bool, Error> {
    let exclude_booking_id = exclude_booking_id.cloned().unwrap_or(BookingId::nil());
    let exclude_adhoc_reservation_id = exclude_adhoc_reservation_id
        .cloned()
        .unwrap_or(AdhocReservationId::nil());

    Ok(sqlx::query!(
        r#"
            SELECT
                (SELECT COUNT(*)
                FROM booking
                WHERE
                    time_to >= ?1 AND
                    MAX(time_from, ?1) < MIN(time_to, ?2) AND
                    (canceled_at IS NULL OR canceled_at > ?4) AND
                    unit_id = ?3 AND
                    id != ?5) +
                (SELECT COUNT(*)
                FROM adhoc_reservation
                WHERE
                    time_to >= ?1 AND
                    MAX(time_from, ?1) < MIN(time_to, ?2) AND
                    (deleted_at IS NULL OR deleted_at > ?4) AND
                    unit_id = ?3 AND
                    id != ?6)
                AS "count"
        "#,
        range_from,
        range_to,
        unit_id,
        now,
        exclude_booking_id,
        exclude_adhoc_reservation_id,
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
    range_from: &DateTime<Utc>,
    range_to: &DateTime<Utc>,
    exclude_booking_id: Option<&BookingId>,
    exclude_adhoc_reservation_id: Option<&AdhocReservationId>,
) -> Result<Vec<OccupiedSlot>, Error> {
    let mut records = vec![];

    let exclude_booking_id = exclude_booking_id.cloned().unwrap_or(BookingId::nil());
    let exclude_adhoc_reservation_id = exclude_adhoc_reservation_id
        .cloned()
        .unwrap_or(AdhocReservationId::nil());

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
            b.time_to >= ?1 AND b.time_from < ?2 AND
            b.unit_id = ?3 AND
            (b.canceled_at IS NULL OR b.canceled_at > ?4) AND
            b.id != ?5
        "#,
        range_from,
        range_to,
        unit_id,
        now,
        exclude_booking_id,
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
        FROM adhoc_reservation
        WHERE
            time_to >= ?1 AND time_from < ?2 AND
            unit_id = ?3 AND
            (deleted_at IS NULL OR deleted_at > ?4) AND
            id != ?5
        "#,
        range_from,
        range_to,
        unit_id,
        now,
        exclude_adhoc_reservation_id,
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

pub async fn get_product(
    connection: &mut SqliteConnection,
    product_id: &ProductId,
) -> Result<Option<Product>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT
            p.discriminator AS "p_discriminator: ProductDiscriminator",
            b.id AS "b_id: Option<BookingId>",
            b.unit_id AS "b_unit_id: Option<UnitId>",
            hu.id AS "hu_id: Option<UserId>",
            hu.provider AS "hu_provider: Option<IdentityProvider>",
            hu.foreign_id AS "hu_foreign_id: Option<String>",
            hu.name AS "hu_name: Option<String>",
            hu.created_at AS "hu_created_at: Option<DateTime<Utc>>",
            hu.deactivated_at AS "hu_deactivated_at: DateTime<Utc>",
            hu.license_plate_number AS "hu_license_plate_number",
            ci.discriminator AS "ci_discriminator: Option<IdentityDiscriminator>",
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
            b.time_from AS "b_time_from: Option<DateTime<Utc>>",
            b.time_to AS "b_time_to: Option<DateTime<Utc>>",
            b.created_at AS "b_created_at: Option<DateTime<Utc>>",
            b.confirmed_at AS "b_confirmed_at: DateTime<Utc>",
            b.canceled_at AS "b_canceled_at: DateTime<Utc>",
            ba.id AS "ba_id: Option<BookingAmendmentId>",
            ba.booking_id AS "ba_booking_id: Option<BookingId>",
            ba.original_time_from AS "ba_original_time_from: Option<DateTime<Utc>>",
            ba.original_time_to AS "ba_original_time_to: Option<DateTime<Utc>>",
            ba.desired_time_from AS "ba_desired_time_from: Option<DateTime<Utc>>",
            ba.desired_time_to AS "ba_desired_time_to: Option<DateTime<Utc>>",
            ba.created_at AS "ba_created_at: Option<DateTime<Utc>>",
            ba.confirmed_at AS "ba_confirmed_at: DateTime<Utc>",
            ba.canceled_at AS "ba_canceled_at: DateTime<Utc>"
        FROM product "p"
        LEFT OUTER JOIN booking "b" ON
            b.id = p.id AND
            p.discriminator = 'booking'
        LEFT OUTER JOIN booking_amendment "ba" ON
            ba.id = p.id AND
            p.discriminator = 'booking_amendment'
        LEFT OUTER JOIN user "hu" ON b.holder_id = hu.id
        LEFT OUTER JOIN identity "ci" ON b.customer_id = ci.id
        LEFT OUTER JOIN user "cu" ON
            ci.discriminator = 'user' AND
            ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON
            ci.discriminator = 'group' AND
            ci.id = cg.id
        WHERE
            p.id = ?1
        "#,
        product_id
    )
    .fetch_optional(&mut *connection)
    .await?;

    Ok(if let Some(result) = result {
        Some(match result.p_discriminator {
            ProductDiscriminator::Booking => Product::Booking(Box::new(Booking {
                id: result.b_id.ok_or(Error::MissingField("b_id"))?,
                unit_id: result.b_unit_id.ok_or(Error::MissingField("b_unit_id"))?,
                holder: User {
                    id: result.hu_id.ok_or(Error::MissingField("hu_id"))?,
                    provider: result
                        .hu_provider
                        .ok_or(Error::MissingField("hu_provider"))?,
                    foreign_id: result
                        .hu_foreign_id
                        .ok_or(Error::MissingField("hu_foreign_id"))?,
                    name: result.hu_name.ok_or(Error::MissingField("hu_name"))?,
                    created_at: result
                        .hu_created_at
                        .ok_or(Error::MissingField("hu_created_at"))?,
                    deactivated_at: result.hu_deactivated_at,
                    license_plate_number: result.hu_license_plate_number,
                },
                customer: match result.ci_discriminator {
                    Some(IdentityDiscriminator::User) => Identity::User(User {
                        id: result.cu_id.ok_or(Error::MissingField("cu_id"))?,
                        provider: result
                            .cu_provider
                            .ok_or(Error::MissingField("cu_provider"))?,
                        foreign_id: result
                            .cu_foreign_id
                            .ok_or(Error::MissingField("cu_foreign_id"))?,
                        name: result.cu_name.ok_or(Error::MissingField("cu_name"))?,
                        created_at: result
                            .cu_created_at
                            .ok_or(Error::MissingField("cu_created_at"))?,
                        deactivated_at: result.cu_deactivated_at,
                        license_plate_number: result.cu_license_plate_number,
                    }),
                    Some(IdentityDiscriminator::Group) => Identity::Group(Group {
                        id: result.cg_id.ok_or(Error::MissingField("cg_id"))?,
                        name: result.cg_name.ok_or(Error::MissingField("cg_name"))?,
                        owner_id: result
                            .cg_owner_id
                            .ok_or(Error::MissingField("cg_owner_id"))?,
                        is_open: result.cg_is_open.ok_or(Error::MissingField("cg_is_open"))?,
                        created_at: result
                            .cg_created_at
                            .ok_or(Error::MissingField("cg_created_at"))?,
                        deleted_at: result.cg_deleted_at,
                    }),
                    None => Err(Error::MissingField("ci_discriminator"))?,
                },
                time_from: result
                    .b_time_from
                    .ok_or(Error::MissingField("b_time_from"))?,
                time_to: result.b_time_to.ok_or(Error::MissingField("b_time_to"))?,
                created_at: result
                    .b_created_at
                    .ok_or(Error::MissingField("b_created_at"))?,
                confirmed_at: result.b_confirmed_at,
                canceled_at: result.b_canceled_at,
            })),
            ProductDiscriminator::BookingAmendment => Product::Amendment(BookingAmendment {
                id: result.ba_id.ok_or(Error::MissingField("ba_id"))?,
                booking_id: result
                    .ba_booking_id
                    .ok_or(Error::MissingField("ba_booking_id"))?,
                original_time_from: result
                    .ba_original_time_from
                    .ok_or(Error::MissingField("ba_original_time_from"))?,
                original_time_to: result
                    .ba_original_time_to
                    .ok_or(Error::MissingField("ba_original_time_to"))?,
                desired_time_from: result
                    .ba_desired_time_from
                    .ok_or(Error::MissingField("ba_desired_time_from"))?,
                desired_time_to: result
                    .ba_desired_time_to
                    .ok_or(Error::MissingField("ba_desired_time_to"))?,
                created_at: result
                    .ba_created_at
                    .ok_or(Error::MissingField("ba_created_at"))?,
                confirmed_at: result.ba_confirmed_at,
                canceled_at: result.ba_canceled_at,
            }),
        })
    } else {
        None
    })
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
    now: &DateTime<Utc>,
    unit_id: &UnitId,
    time_from: &DateTime<Utc>,
    time_to: &DateTime<Utc>,
    confirmed_only: bool,
    exclude_canceled: bool,
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
            b.time_to >= ?2 AND
            b.time_from < ?3
        "#,
        unit_id,
        time_from,
        time_to,
    )
    .fetch_all(&mut *connection)
    .await?
    .into_iter()
    .filter(|v| {
        if confirmed_only && !v.b_confirmed_at.map(|v| &v < now).unwrap_or(false) {
            return false;
        }
        if exclude_canceled && v.b_canceled_at.map(|v| &v < now).unwrap_or(false) {
            return false;
        }

        true
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
    range_from: &DateTime<Utc>,
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
        range_from,
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

pub async fn get_complete_bookings(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    date_from: &DateTime<Utc>,
    date_to: &DateTime<Utc>,
    offset: i64,
    limit: i64,
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
        JOIN product "p" ON b.id = p.id AND p.discriminator = 'booking'
        LEFT OUTER JOIN user "cu" ON ci.discriminator = 'user' AND ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON ci.discriminator = 'group' AND ci.id = cg.id
        WHERE
            b.confirmed_at < ?1 AND
            (b.canceled_at IS NULL OR b.canceled_at >= ?1) AND
            b.time_to < ?1 AND
            b.time_to >= ?2 AND b.time_from < ?3
        ORDER BY b.time_to DESC
        LIMIT ?4 OFFSET ?5
        "#,
        now,
        date_from,
        date_to,
        limit,
        offset,
    )
    .fetch_all(&mut *connection)
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

pub async fn get_bookings_with_pending_cash_payment(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    include_canceled: bool,
    date_from: &DateTime<Utc>,
    date_to: &DateTime<Utc>,
    offset: i64,
    limit: i64,
) -> Result<Vec<(Booking, CashTransaction)>, Error> {
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
            ctx.depositor_name AS "ctx_depsitor_name: String",
            ctx.price AS "ctx_price: i64",
            ctx.created_at AS "ctx_created_at: DateTime<Utc>",
            ctx.confirmed_at AS "ctx_confirmed_at: DateTime<Utc>",
            ctx.refund_account AS "ctx_refund_account: String",
            ctx.refund_price AS "ctx_refund_price: i64",
            ctx.refunded_at AS "ctx_refunded_at: DateTime<Utc>"
        FROM booking "b"
        JOIN user "hu" ON b.holder_id = hu.id
        JOIN identity "ci" ON b.customer_id = ci.id
        JOIN product "p" ON b.id = p.id AND p.discriminator = 'booking'
        JOIN cash_transaction "ctx" ON ctx.product_id = p.id
        LEFT OUTER JOIN user "cu" ON ci.discriminator = 'user' AND ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON ci.discriminator = 'group' AND ci.id = cg.id
        WHERE
            (b.confirmed_at IS NULL OR b.confirmed_at >= ?1) AND
            (ctx.confirmed_at IS NULL OR ctx.confirmed_at >= ?1) AND
            b.time_to >= ?2 AND b.time_from < ?3
        ORDER BY b.created_at DESC
        LIMIT ?4 OFFSET ?5
        "#,
        now,
        date_from,
        date_to,
        limit,
        offset,
    )
    .fetch_all(&mut *connection)
    .await?
    .into_iter()
    .filter(|v| {
        if !include_canceled {
            !is_in_effect(&v.b_canceled_at, now)
        } else {
            true
        }
    })
    .map(|v| {
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
            CashTransaction {
                product_id: v.b_id.into(),
                created_at: v.ctx_created_at,
                depositor_name: v.ctx_depsitor_name,
                price: v.ctx_price,
                confirmed_at: v.ctx_confirmed_at,
                refund_account: v.ctx_refund_account,
                refund_price: v.ctx_refund_price,
                refunded_at: v.ctx_refunded_at,
            },
        ))
    })
    .collect::<Result<Vec<_>, Error>>()
}

pub async fn get_bookings_with_pending_cash_refunds(
    connection: &mut SqliteConnection,
    date_from: &DateTime<Utc>,
    date_to: &DateTime<Utc>,
    offset: i64,
    limit: i64,
) -> Result<Vec<(Booking, CashTransaction)>, Error> {
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
            ctx.depositor_name AS "ctx_depsitor_name: String",
            ctx.price AS "ctx_price: i64",
            ctx.created_at AS "ctx_created_at: DateTime<Utc>",
            ctx.confirmed_at AS "ctx_confirmed_at: DateTime<Utc>",
            ctx.refund_account AS "ctx_refund_account: String",
            ctx.refund_price AS "ctx_refund_price: i64",
            ctx.refunded_at AS "ctx_refunded_at: DateTime<Utc>"
        FROM booking "b"
        JOIN user "hu" ON b.holder_id = hu.id
        JOIN identity "ci" ON b.customer_id = ci.id
        JOIN product "p" ON b.id = p.id AND p.discriminator = 'booking'
        LEFT OUTER JOIN user "cu" ON ci.discriminator = 'user' AND ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON ci.discriminator = 'group' AND ci.id = cg.id
        LEFT OUTER JOIN cash_transaction "ctx" ON p.id = ctx.product_id
        WHERE
            ctx.refund_price IS NOT NULL AND
            ctx.refunded_at IS NULL AND
            b.time_to >= ?1 AND b.time_from < ?2
        ORDER BY b.created_at DESC
        LIMIT ?3 OFFSET ?4
        "#,
        date_from,
        date_to,
        limit,
        offset,
    )
    .fetch_all(&mut *connection)
    .await?
    .into_iter()
    .map(|v| {
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
            CashTransaction {
                product_id: v.b_id.into(),
                created_at: v.ctx_created_at,
                depositor_name: v.ctx_depsitor_name,
                price: v.ctx_price,
                confirmed_at: v.ctx_confirmed_at,
                refund_account: v.ctx_refund_account,
                refund_price: v.ctx_refund_price,
                refunded_at: v.ctx_refunded_at,
            },
        ))
    })
    .collect::<Result<Vec<_>, Error>>()
}

pub async fn get_confirmed_bookings(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    date_from: &DateTime<Utc>,
    date_to: &DateTime<Utc>,
    offset: i64,
    limit: i64,
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
            b.confirmed_at < ?1 AND
            b.time_to >= ?2 AND b.time_from < ?3
        ORDER BY b.time_from DESC
        LIMIT ?4 OFFSET ?5
        "#,
        now,
        date_from,
        date_to,
        limit,
        offset,
    )
    .fetch_all(&mut *connection)
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

pub async fn create_booking(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    unit_id: &UnitId,
    user_id: &UserId,
    customer_id: &IdentityId,
    time_from: &DateTime<Utc>,
    time_to: &DateTime<Utc>,
    is_confirmed: bool,
) -> Result<BookingId, Error> {
    if is_unit_enabled(connection, unit_id).await? != Some(true) {
        return Err(Error::UnitNotFound);
    }

    if !is_booking_available(connection, now, unit_id, time_from, time_to, None, None).await? {
        return Err(Error::TimeRangeOccupied);
    }

    let booking_id = BookingId::generate();
    let product_id = ProductId::from(booking_id);

    sqlx::query!(
        r#"
        INSERT INTO product(id, discriminator)
        VALUES(?1, 'booking')
        "#,
        product_id,
    )
    .execute(&mut *connection)
    .await?;

    let confirmed_at = if is_confirmed { Some(now) } else { None };

    sqlx::query!(
        r#"
        INSERT INTO booking(id, unit_id, holder_id, customer_id, time_from, time_to, created_at, confirmed_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        booking_id,
        unit_id,
        user_id,
        customer_id,
        time_from,
        time_to,
        now,
        confirmed_at,
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

pub async fn update_booking_time(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    booking_id: &BookingId,
    time_from: &DateTime<Utc>,
    time_to: &DateTime<Utc>,
) -> Result<bool, Error> {
    let booking = get_booking(&mut *connection, booking_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    if !is_booking_available(
        &mut *connection,
        now,
        &booking.unit_id,
        time_from,
        time_to,
        Some(booking_id),
        None,
    )
    .await?
    {
        return Err(Error::TimeRangeOccupied);
    }

    if &booking.time_from != time_from || &booking.time_to != time_to {
        let result = sqlx::query!(
            r#"
            UPDATE booking
            SET
                time_from=?1,
                time_to=?2
            WHERE
                id=?3
            "#,
            time_from,
            time_to,
            booking_id,
        )
        .execute(&mut *connection)
        .await?;

        Ok(result.rows_affected() > 0)
    } else {
        Ok(false)
    }
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

pub async fn get_booking_amendment(
    connection: &mut SqliteConnection,
    id: &BookingAmendmentId,
) -> Result<Option<BookingAmendment>, Error> {
    Ok(sqlx::query_as!(
        BookingAmendment,
        r#"
        SELECT
            id AS "id: _",
            booking_id AS "booking_id: _",
            original_time_from AS "original_time_from: _",
            original_time_to AS "original_time_to: _",
            desired_time_from AS "desired_time_from: _",
            desired_time_to AS "desired_time_to: _",
            created_at AS "created_at: _",
            confirmed_at AS "confirmed_at: _",
            canceled_at AS "canceled_at: _"
        FROM
            booking_amendment
        WHERE
            id = ?1
        "#,
        id
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn create_booking_amendment(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    booking_id: &BookingId,
    desired_time_from: &DateTime<Utc>,
    desired_time_to: &DateTime<Utc>,
    is_confirmed: bool,
) -> Result<BookingAmendmentId, Error> {
    let booking = get_booking(&mut *connection, booking_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    if is_unit_enabled(connection, &booking.unit_id).await? != Some(true) {
        return Err(Error::UnitNotFound);
    }

    if !is_booking_available(
        connection,
        now,
        &booking.unit_id,
        desired_time_from,
        desired_time_to,
        Some(booking_id),
        None,
    )
    .await?
    {
        return Err(Error::TimeRangeOccupied);
    }

    let booking_amendment_id = BookingAmendmentId::generate();
    let product_id = ProductId::from(booking_amendment_id);

    sqlx::query!(
        r#"
        INSERT INTO product(id, discriminator)
        VALUES(?1, 'booking_amendment')
        "#,
        product_id,
    )
    .execute(&mut *connection)
    .await?;

    let confirmed_at = if is_confirmed { Some(now) } else { None };

    sqlx::query!(
        r#"
        INSERT INTO booking_amendment(
            id,
            booking_id,
            original_time_from,
            original_time_to,
            desired_time_from,
            desired_time_to,
            created_at,
            confirmed_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        booking_amendment_id,
        booking_id,
        booking.time_from,
        booking.time_to,
        desired_time_from,
        desired_time_to,
        now,
        confirmed_at,
    )
    .execute(&mut *connection)
    .await?;

    Ok(booking_amendment_id)
}

pub async fn confirm_booking_amendment(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    booking_amendment_id: &BookingAmendmentId,
) -> Result<bool, Error> {
    let booking_amendment = get_booking_amendment(&mut *connection, booking_amendment_id)
        .await?
        .ok_or(Error::BookingAmendmentNotFound)?;

    if is_in_effect(&booking_amendment.confirmed_at, now)
        || is_in_effect(&booking_amendment.canceled_at, now)
    {
        return Ok(false);
    }

    let booking = get_booking(&mut *connection, &booking_amendment.booking_id)
        .await?
        .ok_or(Error::BookingNotFound)?;

    if booking.time_from != booking_amendment.desired_time_from
        || booking.time_to != booking_amendment.desired_time_to
    {
        let _ = sqlx::query!(
            r#"
            UPDATE booking
            SET
                time_from=?1,
                time_to=?2
            WHERE id=?3
            "#,
            booking_amendment.desired_time_from,
            booking_amendment.desired_time_to,
            booking_amendment.booking_id
        )
        .execute(&mut *connection)
        .await?;
    }

    let _ = sqlx::query!(
        r#"
        UPDATE booking_amendment
        SET confirmed_at=?1
        WHERE id=?2
        "#,
        now,
        booking_amendment_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(true)
}

pub async fn cancel_booking_amendment(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    booking_amendment_id: &BookingAmendmentId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE booking_amendment
        SET canceled_at=?1
        WHERE
            id=?2 AND
            (confirmed_at IS NULL OR confirmed_at > ?1) AND
            (canceled_at IS NULL OR canceled_at > ?1)
        "#,
        now,
        booking_amendment_id,
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn create_adhoc_reservation(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    unit_id: &UnitId,
    customer_id: &IdentityId,
    user_id: &UserId,
    time_from: &DateTime<Utc>,
    time_to: &DateTime<Utc>,
    remark: &Option<String>,
    deletes_at: &Option<DateTime<Utc>>,
) -> Result<AdhocReservationId, Error> {
    if is_unit_enabled(connection, unit_id).await? != Some(true) {
        return Err(Error::UnitNotFound);
    }

    if !is_booking_available(connection, now, unit_id, time_from, time_to, None, None).await? {
        return Err(Error::TimeRangeOccupied);
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO adhoc_reservation(unit_id, holder_id, customer_id, time_from, time_to, remark, deleted_at)
        VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)
        RETURNING id
        "#,
        unit_id,
        user_id,
        customer_id,
        time_from,
        time_to,
        remark,
        deletes_at,
    )
    .fetch_one(&mut *connection)
    .await?;

    Ok(result.id.into())
}

pub async fn get_adhoc_reservation(
    connection: &mut SqliteConnection,
    id: &AdhocReservationId,
) -> Result<Option<AdhocReservation>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT
            r.id AS "id: AdhocReservationId",
            r.unit_id AS "unit_id: UnitId",
            r.time_from AS "time_from: DateTime<Utc>",
            r.time_to AS "time_to: DateTime<Utc>",
            r.deleted_at AS "deleted_at: DateTime<Utc>",
            r.remark,
            hu.id AS "hu_id: UserId",
            hu.provider AS "hu_provider: IdentityProvider",
            hu.foreign_id AS "hu_foreign_id: String",
            hu.name AS "hu_name: String",
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
            cg.deleted_at AS "cg_deleted_at: DateTime<Utc>"
        FROM adhoc_reservation "r"
        JOIN user "hu" ON r.holder_id = hu.id
        JOIN identity "ci" ON r.customer_id = ci.id
        LEFT OUTER JOIN user "cu" ON ci.discriminator = 'user' AND ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON ci.discriminator = 'group' AND ci.id = cg.id
        WHERE
            r.id = ?1
        "#,
        id
    )
    .fetch_optional(&mut *connection)
    .await?;

    let Some(v) = result else {
        return Ok(None);
    };

    Ok(Some(AdhocReservation {
        id: v.id,
        unit_id: v.unit_id,
        time_from: v.time_from,
        time_to: v.time_to,
        deleted_at: v.deleted_at,
        remark: v.remark,
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
    }))
}

pub async fn delete_adhoc_reservation(
    connection: &mut SqliteConnection,
    reservation_id: AdhocReservationId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM adhoc_reservation WHERE id=?1
        "#,
        reservation_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_adhoc_reservations_by_unit_id(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    unit_id: &UnitId,
    range_from: Option<DateTime<Utc>>,
) -> Result<Vec<AdhocReservation>, Error> {
    let range_from = range_from.unwrap_or_default();

    let result = sqlx::query!(
        r#"
        SELECT
            r.id AS "id: AdhocReservationId",
            r.unit_id AS "unit_id: UnitId",
            r.time_from AS "time_from: DateTime<Utc>",
            r.time_to AS "time_to: DateTime<Utc>",
            r.deleted_at AS "deleted_at: DateTime<Utc>",
            r.remark,
            hu.id AS "hu_id: UserId",
            hu.provider AS "hu_provider: IdentityProvider",
            hu.foreign_id AS "hu_foreign_id: String",
            hu.name AS "hu_name: String",
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
            cg.deleted_at AS "cg_deleted_at: DateTime<Utc>"
        FROM adhoc_reservation "r"
        JOIN user "hu" ON r.holder_id=hu.id
        JOIN identity "ci" ON r.customer_id = ci.id
        LEFT OUTER JOIN user "cu" ON ci.discriminator = 'user' AND ci.id = cu.id
        LEFT OUTER JOIN "group" "cg" ON ci.discriminator = 'group' AND ci.id = cg.id
        WHERE
            unit_id = ?1 AND
            time_to > ?2 AND
            (r.deleted_at IS NULL OR r.deleted_at > ?3)
        ORDER BY r.time_from ASC
        "#,
        unit_id,
        range_from,
        now,
    )
    .fetch_all(&mut *connection)
    .await?;

    result
        .into_iter()
        .map(|v| {
            Ok(AdhocReservation {
                id: v.id,
                unit_id: v.unit_id,
                time_from: v.time_from,
                time_to: v.time_to,
                deleted_at: v.deleted_at,
                remark: v.remark,
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
            })
        })
        .collect::<Result<Vec<_>, Error>>()
}

pub async fn expire_adhoc_reservation(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    adhoc_reservation_id: &AdhocReservationId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE adhoc_reservation SET deleted_at=?1 WHERE id=?2
        "#,
        now,
        adhoc_reservation_id,
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_audio_recording(
    connection: &mut SqliteConnection,
    booking_id: &BookingId,
) -> Result<Option<AudioRecording>, Error> {
    Ok(sqlx::query_as!(
        AudioRecording,
        r#"
        SELECT
            booking_id AS "booking_id: _",
            url,
            created_at AS "created_at: _",
            expires_in AS "expires_in: _"
        FROM audio_recording
        WHERE booking_id=?1
        "#,
        booking_id
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn create_audio_recording(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    booking_id: &BookingId,
    url: &str,
    expires_in: Option<&DateTime<Utc>>,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO audio_recording(booking_id, url, created_at, expires_in)
        VALUES(?1, ?2, ?3, ?4)
        "#,
        booking_id,
        url,
        now,
        expires_in
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_telemetry_file(
    connection: &mut SqliteConnection,
    booking_id: &BookingId,
    r#type: TelemetryType,
) -> Result<Option<TelemetryFile>, Error> {
    Ok(sqlx::query_as!(
        TelemetryFile,
        r#"
        SELECT
            booking_id AS "booking_id: _",
            type AS "type: _",
            file_name,
            uploaded_at AS "uploaded_at: _"
        FROM telemetry_file
        WHERE
            booking_id = ?1 AND
            type = ?2
        "#,
        booking_id,
        r#type
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn get_telemetry_files(
    connection: &mut SqliteConnection,
    booking_id: &BookingId,
) -> Result<Vec<TelemetryFile>, Error> {
    Ok(sqlx::query_as!(
        TelemetryFile,
        r#"
        SELECT
            booking_id AS "booking_id: _",
            type AS "type: _",
            file_name,
            uploaded_at AS "uploaded_at: _"
        FROM telemetry_file
        WHERE
            booking_id = ?1
        "#,
        booking_id
    )
    .fetch_all(&mut *connection)
    .await?)
}

pub async fn create_telemetry_file(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    booking_id: &BookingId,
    r#type: TelemetryType,
    file_name: String,
) -> Result<(), Error> {
    let _ = sqlx::query!(
        r#"
        INSERT INTO telemetry_file(booking_id, type, file_name, uploaded_at)
        VALUES(?1, ?2, ?3, ?4)
        "#,
        booking_id,
        r#type,
        file_name,
        now,
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn create_adhoc_parking(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    space_id: &SpaceId,
    time_from: &DateTime<Utc>,
    time_to: &DateTime<Utc>,
    license_plate_number: &str,
) -> Result<AdhocParkingId, Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO adhoc_parking(space_id, time_from, time_to, license_plate_number, created_at)
        VALUES(?1, ?2, ?3, ?4, ?5)
        RETURNING id AS "id: AdhocParkingId"
        "#,
        space_id,
        time_from,
        time_to,
        license_plate_number,
        now
    )
    .fetch_one(&mut *connection)
    .await?;

    Ok(result.id)
}

pub async fn get_adhoc_parkings(
    connection: &mut SqliteConnection,
    space_id: &SpaceId,
    range_from: Option<DateTime<Utc>>,
    range_to: Option<DateTime<Utc>>,
) -> Result<Vec<AdhocParking>, Error> {
    let range_from = range_from.unwrap_or_default();
    let range_to = range_to.unwrap_or(MAX_ARBITRARY_DATETIME_RANGE);

    Ok(sqlx::query_as!(
        AdhocParking,
        r#"
        SELECT
            id,
            space_id AS "space_id: _",
            time_from AS "time_from: _",
            time_to AS "time_to: _",
            license_plate_number,
            created_at AS "created_at: _"
        FROM adhoc_parking
        WHERE
            space_id = ?1 AND
            time_to >= ?2 AND
            time_from < ?3
        "#,
        space_id,
        range_from,
        range_to,
    )
    .fetch_all(&mut *connection)
    .await?)
}

pub async fn delete_adhoc_parking(
    connection: &mut SqliteConnection,
    id: AdhocParkingId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM adhoc_parking WHERE id=?1
        "#,
        id
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}
