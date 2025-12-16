use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};
use dxe_data::Error as DataError;
use dxe_extern::kakao::client::Error as KakaoClientError;

use crate::config::UrlConfig;
use crate::utils::session::log_out;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("다시 로그인해 주십시오.")]
    LoggedOut(Data<UrlConfig>),
    #[error("권한이 없습니다.")]
    Forbidden,
    #[error("첫 화면으로 돌아가서 다시 로그인을 진행해 주십시오.")]
    InvalidKakaoAccessToken,
    #[error("잘못된 날짜 범위입니다.")]
    InvalidTimeRange,
    #[error("현재 진행중인 예약은 변경할 수 없습니다.")]
    OngoingBookingNotModifiable,
    #[error("입력하신 날짜는 이미 예약이 완료되었습니다.")]
    TimeRangeOccupied,
    #[error("잘못된 요청입니다.")]
    MissingField(&'static str),
    #[error("잘못된 요청입니다.")]
    UnitNotFound,
    #[error("그룹을 찾을 수 없습니다.")]
    GroupNotFound,
    #[error("사용자를 찾을 수 없습니다.")]
    UserNotFound,
    #[error("예약을 찾을 수 없습니다.")]
    BookingNotFound,
    #[error("예약 변경 요청을 찾을 수 없습니다.")]
    BookingAmendmentNotFound,
    #[error("녹음 파일을 찾을 수 없습니다.")]
    AudioRecordingNotFound,
    #[error("이미 그룹에 속해있기 때문에 그룹으로 전환할 수 없습니다.")]
    BookingNotAssignableToGroup,
    #[error("해당 그룹에 속해있지 않습니다.")]
    UserNotMemberOf,
    #[error("그룹에서 나갈 수 없습니다.")]
    CannotLeaveGroup,
    #[error("그룹을 삭제할 수 없습니다.")]
    CannotDeleteGroup,
    #[error("현재 그룹이 열려 있지 않습니다.")]
    GroupIsNotOpen,
    #[error("소유권을 이전하려는 사용자가 그룹에 가입되어 있지 않습니다.")]
    CannotTransferGroupOwnership,
    #[error("환불 계좌 정보를 입력해 주세요.")]
    RefundAccountRequired,
    #[error("환불 시간이 지났습니다.")]
    NotRefundable,
    #[error("현재 이용 시간이 아닙니다.")]
    BookingNotActive,
    #[error("출입구 개방에 실패했습니다.")]
    DoorNotOpened(String),
    #[error("이미 확정된 예약입니다.")]
    BookingAlreadyConfirmed,
    #[error("결제 정보를 찾을 수 없습니다.")]
    ForeignPaymentNotFound,
    #[error("잘못된 파일 업로드입니다.")]
    BadFileUpload,
    #[error("결제에 실패했습니다: {0}")]
    PaymentFailed(String),
    #[error("로그인에 실패했습니다.")]
    AuthFailed,
    #[error("{message}")]
    TossPaymentsFailed { code: String, message: String },
    #[error("인증에 실패했습니다.")]
    Jwt(actix_jwt_auth_middleware::AuthError),
    #[error("카카오 API 에러가 발생했습니다: {0}")]
    Kakao(dxe_extern::kakao::client::KakaoError),
    #[error("외부 API와 통신 중 에러가 발생했습니다.")]
    Http(#[from] reqwest::Error),
    #[error("데이터베이스 오류가 발생했습니다: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[cfg_attr(debug_assertions, error("내부 오류가 발생했습니다: {0}"))]
    #[cfg_attr(not(debug_assertions), error("내부 오류가 발생했습니다."))]
    Internal(Box<dyn std::error::Error>),
}

impl From<actix_jwt_auth_middleware::AuthError> for Error {
    fn from(value: actix_jwt_auth_middleware::AuthError) -> Self {
        Self::Jwt(value)
    }
}

impl From<DataError> for Error {
    fn from(value: DataError) -> Self {
        match value {
            DataError::InvalidTimeRange => Self::InvalidTimeRange,
            DataError::TimeRangeOccupied => Self::TimeRangeOccupied,
            DataError::UnitNotFound => Self::UnitNotFound,
            DataError::UserNotFound => Self::UserNotFound,
            DataError::BookingNotFound => Self::BookingNotFound,
            DataError::BookingAmendmentNotFound => Self::BookingAmendmentNotFound,
            DataError::MissingField(field) => Self::MissingField(field),
            DataError::Sqlx(e) => Self::Sqlx(e),
        }
    }
}

impl From<KakaoClientError> for Error {
    fn from(value: KakaoClientError) -> Self {
        match value {
            KakaoClientError::Http(e) => Self::Http(e),
            KakaoClientError::Kakao(e) => Self::Kakao(e),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::LoggedOut(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::InvalidKakaoAccessToken => StatusCode::BAD_REQUEST,
            Self::InvalidTimeRange => StatusCode::BAD_REQUEST,
            Self::OngoingBookingNotModifiable => StatusCode::BAD_REQUEST,
            Self::TimeRangeOccupied => StatusCode::BAD_REQUEST,
            Self::MissingField(_) => StatusCode::BAD_REQUEST,
            Self::UnitNotFound => StatusCode::BAD_REQUEST,
            Self::GroupNotFound => StatusCode::NOT_FOUND,
            Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::BookingNotFound => StatusCode::NOT_FOUND,
            Self::BookingAmendmentNotFound => StatusCode::NOT_FOUND,
            Self::AudioRecordingNotFound => StatusCode::NOT_FOUND,
            Self::BookingNotAssignableToGroup => StatusCode::BAD_REQUEST,
            Self::UserNotMemberOf => StatusCode::BAD_REQUEST,
            Self::CannotLeaveGroup => StatusCode::BAD_REQUEST,
            Self::CannotDeleteGroup => StatusCode::BAD_REQUEST,
            Self::GroupIsNotOpen => StatusCode::BAD_REQUEST,
            Self::CannotTransferGroupOwnership => StatusCode::BAD_REQUEST,
            Self::RefundAccountRequired => StatusCode::BAD_REQUEST,
            Self::NotRefundable => StatusCode::BAD_REQUEST,
            Self::BookingNotActive => StatusCode::BAD_REQUEST,
            Self::DoorNotOpened(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BookingAlreadyConfirmed => StatusCode::BAD_REQUEST,
            Self::ForeignPaymentNotFound => StatusCode::NOT_FOUND,
            Self::BadFileUpload => StatusCode::BAD_REQUEST,
            Self::PaymentFailed(_) => StatusCode::BAD_REQUEST,
            Self::AuthFailed => StatusCode::FORBIDDEN,
            Self::TossPaymentsFailed { .. } => StatusCode::BAD_REQUEST,
            Self::Jwt(_) => StatusCode::UNAUTHORIZED,
            Self::Kakao(_) => StatusCode::UNAUTHORIZED,
            Self::Http(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let r#type = match self {
            Self::LoggedOut(_) => "LoggedOut",
            Self::Forbidden => "Forbidden",
            Self::InvalidKakaoAccessToken => "InvalidKakaoAccessToken",
            Self::InvalidTimeRange => "InvalidTimeRange",
            Self::TimeRangeOccupied => "TimeRangeOccupied",
            Self::OngoingBookingNotModifiable => "OngoingBookingNotModifiable",
            Self::MissingField(_) => "MissingField",
            Self::UnitNotFound => "UnitNotFound",
            Self::GroupNotFound => "GroupNotFound",
            Self::UserNotFound => "UserNotFound",
            Self::BookingNotFound => "BookingNotFound",
            Self::BookingAmendmentNotFound => "BookingAmendmentNotFound",
            Self::AudioRecordingNotFound => "AudioRecordingNotFound",
            Self::BookingNotAssignableToGroup => "BookingNotAssignableToGroup",
            Self::UserNotMemberOf => "UserNotMemberOf",
            Self::CannotLeaveGroup => "CannotLeaveGroup",
            Self::CannotDeleteGroup => "CannotDeleteGroup",
            Self::GroupIsNotOpen => "GroupIsNotOpen",
            Self::CannotTransferGroupOwnership => "CannotTransferGroupOwnership",
            Self::RefundAccountRequired => "RefundAccountRequired",
            Self::NotRefundable => "NotRefundable",
            Self::BookingNotActive => "BookingNotActive",
            Self::DoorNotOpened(_) => "DoorNotOpened",
            Self::BookingAlreadyConfirmed => "BookingAlreadyConfirmed",
            Self::ForeignPaymentNotFound => "ForeignPaymentNotFound",
            Self::BadFileUpload => "BadFileUpload",
            Self::PaymentFailed(_) => "PaymentFailed",
            Self::AuthFailed => "AuthError",
            Self::TossPaymentsFailed { .. } => "TossPaymentsFailed",
            Self::Jwt(_) => "AuthError",
            Self::Http(_) => "HttpError",
            Self::Kakao(_) => "KakaoApiError",
            Self::Sqlx(_) => "DatabaseError",
            Self::Internal(_) => "InternalError",
        };
        let extras = match self {
            Self::TossPaymentsFailed { code, .. } => {
                Some(serde_json::json!({ code: code.clone() }))
            }
            _ => None,
        };

        let payload = serde_json::json!({
            "type": r#type,
            "message": format!("{self}"),
            "extras": extras,
        });

        let mut response = HttpResponseBuilder::new(self.status_code());

        if let Self::LoggedOut(url_config) = self {
            log_out(&mut response, url_config);
        }

        response.json(payload)
    }
}
