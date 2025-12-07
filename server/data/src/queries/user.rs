use chrono::{DateTime, Utc};
use dxe_types::{IdentityId, IdentityProvider, UserId};
use sqlx::{Executor, QueryBuilder, SqliteConnection};

use crate::Error;
use crate::entities::{IdentityDiscriminator, User, UserCashPaymentInformation};

pub async fn create_user(
    connection: &mut SqliteConnection,
    now: DateTime<Utc>,
    provider: IdentityProvider,
    foreign_id: &str,
    name: &str,
    license_plate_number: Option<&str>,
) -> Result<UserId, Error> {
    let id = IdentityId::generate();
    let user_id: UserId = id.into();

    let _ = sqlx::query!(
        r#"
        INSERT INTO identity(id, discriminator)
        VALUES(?1, ?2)
        "#,
        id,
        IdentityDiscriminator::User,
    )
    .execute(&mut *connection)
    .await?;

    let _ = sqlx::query!(
        r#"
        INSERT INTO user(id, provider, foreign_id, name, created_at, license_plate_number)
        VALUES(?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        user_id,
        provider,
        foreign_id,
        name,
        now,
        license_plate_number
    )
    .execute(&mut *connection)
    .await?;

    Ok(user_id)
}

pub async fn get_user_by_id(
    connection: &mut SqliteConnection,
    user_id: &UserId,
    now: &DateTime<Utc>,
) -> Result<Option<User>, Error> {
    Ok(sqlx::query_as!(
        User,
        r#"
        SELECT
            id AS "id: _",
            provider AS "provider: _",
            foreign_id,
            name,
            created_at AS "created_at: _",
            deactivated_at AS "deactivated_at: _",
            license_plate_number,
            use_pg_payment
        FROM user
        WHERE id = ?1 AND
            (deactivated_at IS NULL OR deactivated_at < ?2)
        "#,
        user_id,
        now
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn get_user_by_foreign_id(
    executor: &mut SqliteConnection,
    provider: IdentityProvider,
    foreign_id: &str,
    now: DateTime<Utc>,
) -> Result<Option<User>, Error> {
    Ok(sqlx::query_as!(
        User,
        r#"
        SELECT
            id AS "id: _",
            provider AS "provider: _",
            foreign_id,
            name,
            created_at AS "created_at: _",
            deactivated_at AS "deactivated_at: _",
            license_plate_number,
            use_pg_payment
        FROM user
        WHERE provider = ?1 AND foreign_id = ?2 AND
            (deactivated_at IS NULL OR deactivated_at < ?3)
        "#,
        provider,
        foreign_id,
        now,
    )
    .fetch_optional(executor)
    .await?)
}

pub async fn get_users(executor: &mut SqliteConnection) -> Result<Vec<User>, Error> {
    Ok(sqlx::query_as!(
        User,
        r#"
        SELECT
            id AS "id: _",
            provider AS "provider: _",
            foreign_id,
            name,
            created_at AS "created_at: _",
            deactivated_at AS "deactivated_at: _",
            license_plate_number,
            use_pg_payment
        FROM user
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(executor)
    .await?)
}

pub async fn update_user(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    user_id: &UserId,
    new_name: &Option<String>,
    new_license_plate_number: &Option<String>,
) -> Result<User, Error> {
    let mut query = QueryBuilder::<'_, sqlx::Sqlite>::new("UPDATE user SET ");
    let mut count = 0;

    if let Some(new_name) = new_name {
        query.push("name=");
        query.push_bind(new_name);
        count += 1;
    }

    if let Some(new_license_plate_number) = new_license_plate_number {
        if count > 0 {
            query.push(", ");
        }

        query.push("license_plate_number=");
        if new_license_plate_number.is_empty() {
            query.push("NULL");
        } else {
            query.push_bind(new_license_plate_number);
        }
    }

    query.push(" WHERE id=");
    query.push_bind(user_id);

    connection.execute(query.build()).await?;

    get_user_by_id(&mut *connection, user_id, now)
        .await?
        .ok_or(Error::UserNotFound)
}

pub async fn is_administrator(
    connection: &mut SqliteConnection,
    user_id: &UserId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        SELECT id
        FROM administrator
        WHERE id=?1
        "#,
        user_id
    )
    .fetch_optional(&mut *connection)
    .await?;

    Ok(result.is_some())
}

pub async fn get_user_cash_payment_information(
    connection: &mut SqliteConnection,
    user_id: &UserId,
) -> Result<Option<UserCashPaymentInformation>, Error> {
    Ok(sqlx::query_as!(
        UserCashPaymentInformation,
        r#"
        SELECT
            user_id AS "user_id: _",
            depositor_name,
            refund_account
        FROM user_cash_payment_information
        WHERE user_id=?1
        "#,
        user_id
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn update_user_cash_payment_depositor_name(
    connection: &mut SqliteConnection,
    user_id: &UserId,
    depositor_name: Option<&str>,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO user_cash_payment_information(user_id, depositor_name)
        VALUES(?1, ?2)
        ON CONFLICT(user_id) DO UPDATE SET
            depositor_name=?2
        "#,
        user_id,
        depositor_name
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn update_user_cash_payment_refund_account(
    connection: &mut SqliteConnection,
    user_id: &UserId,
    refund_account: Option<&str>,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO user_cash_payment_information(user_id, refund_account)
        VALUES(?1, ?2)
        ON CONFLICT(user_id) DO UPDATE SET
            refund_account=?2
        "#,
        user_id,
        refund_account
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}
