use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorV1 {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Ready,
    InProgress,
    WaitingForDeposit,
    Done,
    Canceled,
    PartialCanceled,
    Aborted,
    Expired,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum PaymentMethod {
    #[serde(rename = "카드")]
    CreditCard,
    #[serde(rename = "가상계좌")]
    VirtualAccount,
    #[serde(rename = "간편결제")]
    EasyPay,
    #[serde(rename = "휴대폰")]
    Mobile,
    #[serde(rename = "계좌이체")]
    WireTransfer,
    #[serde(rename = "문화상품권")]
    Voucher,
    #[serde(rename = "도서문화상품권")]
    BookVoucher,
    #[serde(rename = "게임문화상품권")]
    GameVoucher,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum CreditCardType {
    #[serde(rename = "신용")]
    CreditCard,
    #[serde(rename = "체크")]
    DebitCard,
    #[serde(rename = "기프트")]
    GiftCard,
    #[serde(rename = "미확인")]
    Unknown,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum CreditCardOwnerType {
    #[serde(rename = "개인")]
    Personal,
    #[serde(rename = "법인")]
    Corporate,
    #[serde(rename = "미확인")]
    Unknown,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreditCardAcquisitionStatus {
    Ready,
    Requested,
    Completed,
    CancelRequested,
    Canceled,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreditCardInterestPayer {
    Buyer,
    CardCompany,
    Merchant,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RefundStatus {
    None,
    Pending,
    Failed,
    PartialFailed,
    Completed,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SettlementStatus {
    Incompleted,
    Completed,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    Confirm,
    Cancel,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CashReceiptIssueStatus {
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum VirtualAccountType {
    #[serde(rename = "일반")]
    Normal,
    #[serde(rename = "고정")]
    Fixed,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum CashReceiptType {
    #[serde(rename = "소득공제")]
    IncomeDeduction,
    #[serde(rename = "지출증빙")]
    ProofOfExpenditure,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccount {
    pub bank_code: String,
    pub account_number: String,
    pub holder_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cancellation {
    pub cancel_amount: f64,
    pub cancel_reason: String,
    pub tax_free_amount: f64,
    pub tax_exemption_amount: i64,
    pub refundable_amount: f64,
    pub card_discount_amount: f64,
    pub transfer_discount_amount: f64,
    pub easy_pay_discount_amount: f64,
    pub canceled_at: DateTime<FixedOffset>,
    pub transaction_key: String,
    pub receipt_key: Option<String>,
    pub cancel_status: String,
    pub cancel_request_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditCardPayment {
    pub amount: f64,
    pub issuer_code: String,
    pub acquirer_code: Option<String>,
    pub number: String,
    pub installment_plan_months: i32,
    pub approve_no: String,
    pub use_card_point: bool,
    pub card_type: CreditCardType,
    pub owner_type: CreditCardOwnerType,
    pub acquire_status: CreditCardAcquisitionStatus,
    pub is_interest_free: bool,
    pub interest_payer: Option<CreditCardInterestPayer>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualAccountPayment {
    pub account_type: VirtualAccountType,
    pub account_number: String,
    pub bank_code: String,
    pub customer_name: String,
    pub depositor_name: String,
    pub due_date: NaiveDateTime,
    pub refund_status: RefundStatus,
    pub expired: bool,
    pub settlement_status: SettlementStatus,
    pub refund_receive_account: Option<BankAccount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MobilePhonePayment {
    pub customer_mobile_phone: String,
    pub settlement_status: SettlementStatus,
    pub receipt_url: Url,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftCertificatePayment {
    pub approve_no: String,
    pub settlement_status: SettlementStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WireTransferPayment {
    pub bank_code: String,
    pub settlement_status: SettlementStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EasyPayPayment {
    pub provider: String,
    pub amount: f64,
    pub discount_amount: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub url: Url,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Checkout {
    pub url: Url,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CashReceipt {
    pub r#type: CashReceiptType,
    pub receipt_key: String,
    pub issue_number: String,
    pub receipt_url: Url,
    pub amount: f64,
    pub tax_free_amount: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Discount {
    pub amount: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CashReceiptHistory {
    pub receipt_key: String,
    pub order_id: String,
    pub order_name: String,
    pub r#type: CashReceiptType,
    pub issue_number: String,
    pub receipt_url: Url,
    pub business_number: String,
    pub transaction_type: TransactionType,
    pub amount: i64,
    pub tax_free_amount: i64,
    pub issue_status: CashReceiptIssueStatus,
    pub failure: Option<ErrorV1>,
    pub customer_identity_number: String,
    pub requested_at: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefundReceiveAccount {
    pub bank: String,
    pub account_number: String,
    pub holder_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    pub m_id: String,
    pub version: String,
    pub payment_key: String,
    pub order_id: String,
    pub order_name: String,
    pub currency: String,
    pub status: PaymentStatus,
    pub method: Option<PaymentMethod>,
    pub total_amount: i64,
    pub balance_amount: i64,
    pub requested_at: DateTime<FixedOffset>,
    pub approved_at: Option<DateTime<FixedOffset>>,
    pub use_escrow: bool,
    pub last_transaction_key: Option<String>,
    pub supplied_amount: i64,
    pub vat: i64,
    pub culture_expense: bool,
    pub tax_free_amount: i64,
    pub tax_exemption_amount: i64,
    pub cancels: Option<Vec<Cancellation>>,
    pub is_partial_cancelable: bool,
    pub card: Option<CreditCardPayment>,
    pub virtual_account: Option<VirtualAccountPayment>,
    pub secret: Option<String>,
    pub mobile_phone: Option<MobilePhonePayment>,
    pub gift_certificate: Option<GiftCertificatePayment>,
    pub transfer: Option<WireTransferPayment>,
    pub metadata: Option<HashMap<String, String>>,
    pub receipt: Option<Receipt>,
    pub checkout: Option<Checkout>,
    pub easy_pay: Option<EasyPayPayment>,
    pub country: String,
    pub failure: Option<ErrorV1>,
    pub cash_receipt: Option<CashReceipt>,
    pub cash_receipts: Option<Vec<CashReceiptHistory>>,
    pub discount: Option<Discount>,
}
