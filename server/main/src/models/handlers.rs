use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use dxe_types::{IdentityId, UnitId, UserId};
use serde::{Deserialize, Serialize};

use crate::models::entities::{
    Booking, BookingCashPaymentStatus, BookingWithPayments, Group, GroupWithUsers, OccupiedSlot,
    SelfUser,
};

pub mod admin {
    use crate::models::entities::Reservation;

    use super::*;

    #[derive(Debug, Deserialize)]
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
        pub cash_payment_status: Option<BookingCashPaymentStatus>,
    }

    #[derive(Debug, Deserialize)]
    pub struct GetReservationsQuery {
        pub unit_id: UnitId,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetReservationsResponse {
        pub reservations: Vec<Reservation>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateReservationRequest {
        pub unit_id: UnitId,
        pub time_from: DateTime<FixedOffset>,
        pub desired_hours: i64,
        pub temporary: bool,
        pub remark: Option<String>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateReservationResponse {
        pub reservation: Reservation,
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
}

pub mod booking {
    use crate::models::entities::AudioRecording;

    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct CalendarQuery {
        pub unit_id: UnitId,
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
        pub cash_payment_status: BookingCashPaymentStatus,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetBookingResponse {
        pub booking: Booking,
        pub cash_payment_status: Option<BookingCashPaymentStatus>,
    }

    #[derive(Debug, Deserialize)]
    pub struct CancelBookingRequest {
        pub refund_account: Option<String>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CancelBookingResponse {
        pub cash_payment_status: Option<BookingCashPaymentStatus>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AmendBookingRequest {
        pub new_identity_id: Option<IdentityId>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AmendBookingResponse {
        pub booking: Booking,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetAudioRecordingResponse {
        pub audio_recording: Option<AudioRecording>,
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
