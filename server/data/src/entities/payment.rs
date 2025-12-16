use chrono::{DateTime, Utc};
use dxe_types::{AdhocReservationId, ForeignPaymentId, ProductId, UserId};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct CashTransaction {
    pub product_id: ProductId,
    pub depositor_name: String,
    pub price: i64,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub refund_price: Option<i64>,
    pub refund_account: Option<String>,
    pub refunded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TossPaymentsTransaction {
    pub id: ForeignPaymentId,
    pub user_id: UserId,
    pub temporary_reservation_id: Option<AdhocReservationId>,
    pub product_id: Option<ProductId>,
    pub price: i64,
    pub payment_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub refund_price: Option<i64>,
    pub refunded_at: Option<DateTime<Utc>>,
}
