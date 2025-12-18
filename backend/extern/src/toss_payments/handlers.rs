use serde::Serialize;

use crate::toss_payments::types::RefundReceiveAccount;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ConfirmPaymentRequest {
    pub order_id: String,
    pub amount: i64,
    pub payment_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CancelPaymentRequest {
    pub cancel_reason: String,
    pub cancel_amount: Option<i64>,
    pub tax_free_amount: Option<f64>,
    pub currency: Option<String>,
    pub refund_receive_account: Option<RefundReceiveAccount>,
}
