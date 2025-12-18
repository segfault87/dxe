use chrono::{DateTime, Utc};
use dxe_types::{AdhocReservationId, BookingId, ForeignPaymentId, ProductId, UserId};
use sqlx::SqliteConnection;

use crate::Error;
use crate::entities::{CashTransaction, TossPaymentsTransaction};

pub async fn create_cash_transaction(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    product_id: &ProductId,
    depositor_name: &str,
    price: i64,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO cash_transaction(product_id, depositor_name, price, created_at)
        VALUES(?1, ?2, ?3, ?4)
        "#,
        product_id,
        depositor_name,
        price,
        now,
    )
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn get_cash_transaction(
    connection: &mut SqliteConnection,
    product_id: &ProductId,
) -> Result<Option<CashTransaction>, Error> {
    Ok(sqlx::query_as!(
        CashTransaction,
        r#"
        SELECT
            product_id AS "product_id: _",
            depositor_name AS "depositor_name: _",
            price AS "price: _",
            created_at AS "created_at: _",
            confirmed_at AS "confirmed_at: _",
            refund_price AS "refund_price: _",
            refund_account AS "refund_account: _",
            refunded_at AS "refunded_at: _"
        FROM cash_transaction
        WHERE product_id=?1
        "#,
        product_id
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn confirm_cash_payment(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    product_id: &ProductId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE cash_transaction
        SET confirmed_at=?1
        WHERE product_id=?2 AND confirmed_at IS NULL
        "#,
        now,
        product_id,
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn update_cash_refund_information(
    connection: &mut SqliteConnection,
    product_id: &ProductId,
    refund_price: i64,
    refund_account: Option<String>,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE cash_transaction
        SET refund_price=?1, refund_account=?2
        WHERE product_id=?3 AND refunded_at IS NULL
        "#,
        refund_price,
        refund_account,
        product_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn refund_cash_payment(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    product_id: &ProductId,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE cash_transaction
        SET refunded_at=?1
        WHERE product_id=?2 AND refunded_at IS NULL
        "#,
        now,
        product_id
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn create_toss_payments_transaction(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    id: &ForeignPaymentId,
    user_id: &UserId,
    temporary_reservation_id: Option<&AdhocReservationId>,
    product_id: Option<&ProductId>,
    price: i64,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO toss_payments_transaction(id, user_id, temporary_reservation_id, product_id, price, created_at)
        VALUES(?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        id,
        user_id,
        temporary_reservation_id,
        product_id,
        price,
        now,
    ).execute(&mut *connection).await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_toss_payments_transaction_by_id(
    connection: &mut SqliteConnection,
    id: &ForeignPaymentId,
) -> Result<Option<TossPaymentsTransaction>, Error> {
    Ok(sqlx::query_as!(
        TossPaymentsTransaction,
        r#"
        SELECT
            id AS "id: _",
            user_id AS "user_id: _",
            temporary_reservation_id AS "temporary_reservation_id: _",
            product_id AS "product_id: _",
            price,
            payment_key,
            created_at AS "created_at: _",
            confirmed_at AS "confirmed_at: _",
            refund_price,
            refunded_at AS "refunded_at: _"
        FROM
            toss_payments_transaction
        WHERE
            id=?1
        "#,
        id
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn get_toss_payments_transactions_by_booking_amentments(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    booking_id: &BookingId,
) -> Result<Vec<TossPaymentsTransaction>, Error> {
    Ok(sqlx::query_as!(
        TossPaymentsTransaction,
        r#"
        SELECT
            t.id AS "id: _",
            t.user_id AS "user_id: _",
            t.temporary_reservation_id AS "temporary_reservation_id: _",
            t.product_id AS "product_id: _",
            t.price AS "price",
            t.payment_key AS "payment_key",
            t.created_at AS "created_at: _",
            t.confirmed_at AS "confirmed_at: _",
            t.refund_price AS "refund_price",
            t.refunded_at AS "refunded_at: _"
        FROM
            toss_payments_transaction "t"
        JOIN product "p" ON
            t.product_id = p.id AND
            p.discriminator = 'booking_amendment'
        JOIN booking_amendment "ba" ON
            ba.id = p.id
        WHERE
            ba.booking_id = ?1 AND
            t.confirmed_at < ?2 AND
            (t.refunded_at IS NULL OR t.refunded_at > ?2)
        "#,
        booking_id,
        now
    )
    .fetch_all(&mut *connection)
    .await?)
}

pub async fn get_toss_payments_transaction_by_temporary_reservation_id(
    connection: &mut SqliteConnection,
    temporary_reservation_id: &AdhocReservationId,
) -> Result<Option<TossPaymentsTransaction>, Error> {
    Ok(sqlx::query_as!(
        TossPaymentsTransaction,
        r#"
        SELECT
            id AS "id: _",
            user_id AS "user_id: _",
            temporary_reservation_id AS "temporary_reservation_id: _",
            product_id AS "product_id: _",
            price,
            payment_key,
            created_at AS "created_at: _",
            confirmed_at AS "confirmed_at: _",
            refund_price,
            refunded_at AS "refunded_at: _"
        FROM
            toss_payments_transaction
        WHERE
            temporary_reservation_id=?1
        "#,
        temporary_reservation_id,
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn get_toss_payments_transaction_by_product_id(
    connection: &mut SqliteConnection,
    product_id: &ProductId,
) -> Result<Option<TossPaymentsTransaction>, Error> {
    Ok(sqlx::query_as!(
        TossPaymentsTransaction,
        r#"
        SELECT
            id AS "id: _",
            user_id AS "user_id: _",
            temporary_reservation_id AS "temporary_reservation_id: _",
            product_id AS "product_id: _",
            price,
            payment_key,
            created_at AS "created_at: _",
            confirmed_at AS "confirmed_at: _",
            refund_price,
            refunded_at AS "refunded_at: _"
        FROM
            toss_payments_transaction
        WHERE
            product_id=?1
        "#,
        product_id,
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn confirm_toss_payments_transaction(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    id: &ForeignPaymentId,
    product_id: &ProductId,
    payment_key: &str,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE toss_payments_transaction
        SET product_id=?1, confirmed_at=?2, payment_key=?3
        WHERE id=?4
        "#,
        product_id,
        now,
        payment_key,
        id,
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn refund_toss_payments(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    id: &ForeignPaymentId,
    refund_amount: i64,
) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        UPDATE toss_payments_transaction
        SET refund_price=?1, refunded_at=?2
        WHERE id=?3
        "#,
        refund_amount,
        now,
        id,
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}
