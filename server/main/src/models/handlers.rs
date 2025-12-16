use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use dxe_types::{
    AdhocReservationId, BookingId, ForeignPaymentId, IdentityId, SpaceId, UnitId, UserId,
};
use serde::{Deserialize, Serialize};

use crate::models::entities::{
    AdhocParking, AdhocReservation, AudioRecording, Booking, BookingWithPayments, CashTransaction,
    Group, GroupWithUsers, OccupiedSlot, ProductType, SelfUser, Transaction,
};

pub mod admin {
    use super::*;

    #[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum GetBookingsType {
        Confirmed,
        Pending,
        RefundPending,
        Canceled,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub struct GetBookingsQuery {
        pub r#type: GetBookingsType,
        pub date_from: Option<DateTime<FixedOffset>>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetBookingsResponse {
        pub bookings: Vec<BookingWithPayments>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum ModifyAction {
        Confirm,
        Refund,
        Cancel,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ModifyBookingRequest {
        pub action: ModifyAction,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ModifyBookingResponse {
        pub booking: Booking,
        pub cash_transaction: Option<CashTransaction>,
    }

    #[derive(Debug, Deserialize)]
    pub struct GetAdhocReservationsQuery {
        pub unit_id: UnitId,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetAdhocReservationsResponse {
        pub reservations: Vec<AdhocReservation>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateAdhocReservationRequest {
        pub unit_id: UnitId,
        pub customer_id: IdentityId,
        pub time_from: DateTime<FixedOffset>,
        pub desired_hours: i64,
        pub expires_at: Option<DateTime<FixedOffset>>,
        pub remark: Option<String>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateAdhocReservationResponse {
        pub reservation: AdhocReservation,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetGroupsResponse {
        pub groups: Vec<GroupWithUsers>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetUsersResponse {
        pub users: Vec<SelfUser>,
    }

    #[derive(Debug, Deserialize)]
    pub struct GetAdhocParkingsQuery {
        pub space_id: SpaceId,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetAdhocParkingsResponse {
        pub parkings: Vec<AdhocParking>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateAdhocParkingRequest {
        pub space_id: SpaceId,
        pub time_from: DateTime<FixedOffset>,
        pub desired_hours: i64,
        pub license_plate_number: String,
    }
}

pub mod auth {
    use super::*;

    #[derive(Clone, Debug, Deserialize)]
    pub struct KakaoAuthRedirectQuery {
        pub code: Option<String>,
        pub state: Option<String>,
        pub error: Option<String>,
        pub error_description: Option<String>,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct KakaoAuthRegisterRequest {
        pub name: String,
        pub license_plate_number: Option<String>,
    }

    #[derive(Clone, Debug, Deserialize)]
    pub struct HandleAuthQuery {
        pub redirect_to: Option<String>,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HandleAuthRequest {
        pub handle: String,
        pub password: String,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HandleAuthResponse {
        pub redirect_to: String,
    }
}

pub mod booking {
    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct CalendarQuery {
        pub unit_id: UnitId,
        pub exclude_booking_id: Option<BookingId>,
        pub exclude_adhoc_reservation_id: Option<AdhocReservationId>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CalendarResponse {
        pub start: DateTime<FixedOffset>,
        pub end: DateTime<FixedOffset>,
        pub max_booking_hours: i64,
        pub slots: Vec<OccupiedSlot>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CheckRequest {
        pub unit_id: UnitId,
        pub time_from: DateTime<FixedOffset>,
        pub desired_hours: i64,
        pub additional_hours: Option<i64>,
        pub exclude_booking_id: Option<BookingId>,
        pub exclude_adhoc_reservation_id: Option<AdhocReservationId>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CheckResponse {
        pub total_price: i64,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SubmitBookingRequest {
        pub unit_id: UnitId,
        pub time_from: DateTime<FixedOffset>,
        pub desired_hours: i64,
        pub identity_id: IdentityId,
        pub depositor_name: String,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SubmitBookingResponse {
        pub booking: Booking,
        pub cash_transaction: CashTransaction,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetBookingResponse {
        pub booking: Booking,
        pub transaction: Option<Transaction>,
        pub amendable: bool,
        pub extendable_hours: i64,
    }

    #[derive(Debug, Deserialize)]
    pub struct CancelBookingRequest {
        pub refund_account: Option<String>,
        pub cancel_reason: Option<String>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CancelBookingResponse {
        pub transaction: Option<Transaction>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AmendBookingRequest {
        pub new_identity_id: Option<IdentityId>,
        pub new_time_from: Option<DateTime<FixedOffset>>,
        pub additional_hours: Option<i64>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AmendBookingResponse {
        pub booking: Booking,
        pub foreign_payment_id: Option<ForeignPaymentId>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetAudioRecordingResponse {
        pub audio_recording: Option<AudioRecording>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InitiateTossPaymentRequest {
        pub temporary_reservation_id: Option<AdhocReservationId>,
        pub unit_id: UnitId,
        pub time_from: DateTime<FixedOffset>,
        pub desired_hours: i64,
        pub identity_id: IdentityId,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InitiateTossPaymentResponse {
        pub order_id: ForeignPaymentId,
        pub price: i64,
        pub temporary_reservation_id: AdhocReservationId,
        pub expires_in: DateTime<FixedOffset>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ConfirmTossPaymentRequest {
        pub payment_key: String,
        pub order_id: ForeignPaymentId,
        pub amount: i64,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ConfirmTossPaymentResponse {
        pub booking_id: BookingId,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetTossPaymentStateResponse {
        pub r#type: ProductType,
        pub time_from: DateTime<FixedOffset>,
        pub time_to: DateTime<FixedOffset>,
    }
}

pub mod user {
    use super::*;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetGroupResponse {
        pub group: GroupWithUsers,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AmendGroupRequest {
        pub new_name: Option<String>,
        pub new_owner: Option<UserId>,
        pub is_open: Option<bool>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AmendGroupResponse {
        pub group: Group,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateGroupRequest {
        pub name: String,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateGroupResponse {
        pub group: GroupWithUsers,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ListGroupsResponse {
        pub groups: Vec<GroupWithUsers>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MeResponse {
        pub user: SelfUser,
        pub active_bookings: HashMap<UnitId, Booking>,
        pub pending_bookings: HashMap<UnitId, Vec<Booking>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UpdateMeRequest {
        pub new_name: Option<String>,
        pub new_license_plate_number: Option<String>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct UpdateMeResponse {
        pub user: SelfUser,
    }
}
