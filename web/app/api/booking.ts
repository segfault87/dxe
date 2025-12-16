import API from "../api";
import type {
  AmendBookingResponse,
  AmendBookingRequest,
  CalendarResponse,
  CancelBookingResponse,
  CheckRequest,
  CheckResponse,
  GetAudioRecordingResponse,
  GetBookingResponse,
  GetTossPaymentStateResponse,
  SubmitBookingRequest,
  SubmitBookingResponse,
  TossPaymentConfirmRequest,
  TossPaymentConfirmResponse,
  TossPaymentInitiateRequest,
  TossPaymentInitiateResponse,
} from "../types/handlers/booking";
import type {
  AdhocReservationId,
  BookingId,
  ForeignPaymentId,
  UnitId,
} from "../types/models/base";

const calendar = (
  unitId: UnitId,
  excludeBookingId?: BookingId,
  excludeAdhocReservationId?: AdhocReservationId,
) => {
  let url = `/bookings/calendar?unit_id=${unitId}`;
  if (excludeBookingId) {
    url += `&exclude_booking_id=${excludeBookingId}`;
  }
  if (excludeAdhocReservationId) {
    url += `&exclude_adhoc_reservation_id=${excludeAdhocReservationId}`;
  }

  return API.get<CalendarResponse>(url);
};

const check = (data: CheckRequest) => {
  return API.post<CheckResponse>("/bookings/check", data);
};

const submitBooking = (data: SubmitBookingRequest) => {
  return API.post<SubmitBookingResponse>("/bookings", data);
};

const get = (bookingId: BookingId) => {
  return API.get<GetBookingResponse>(`/booking/${bookingId}`);
};

const cancel = (
  bookingId: BookingId,
  refundAccount: string | null = null,
  cancelReason: string | null = null,
) => {
  if (refundAccount !== null) {
    return API.delete<CancelBookingResponse>(
      `/booking/${bookingId}?refund_account=${refundAccount}`,
    );
  } else if (cancelReason !== null) {
    return API.delete<CancelBookingResponse>(
      `/booking/${bookingId}?cancel_reason=${cancelReason}`,
    );
  } else {
    return API.delete<CancelBookingResponse>(`/booking/${bookingId}`);
  }
};

const amend = (bookingId: BookingId, data: AmendBookingRequest) => {
  return API.put<AmendBookingResponse>(`/booking/${bookingId}`, data);
};

const openDoor = (bookingId: BookingId) => {
  return API.post(`/booking/${bookingId}/open`);
};

const getAudioRecording = (bookingId: BookingId) => {
  return API.get<GetAudioRecordingResponse>(`/booking/${bookingId}/recording`);
};

const initiateTossPayment = (data: TossPaymentInitiateRequest) => {
  return API.post<TossPaymentInitiateResponse>("/payments/toss", data);
};

const confirmTossPayment = (data: TossPaymentConfirmRequest) => {
  return API.post<TossPaymentConfirmResponse>("/payments/toss/confirm", data);
};

const cancelTossPayment = (id: ForeignPaymentId) => {
  return API.delete(`/payments/toss/order/${id}`);
};

const getTossPaymentState = (id: ForeignPaymentId) => {
  return API.get<GetTossPaymentStateResponse>(`/payments/toss/order/${id}`);
};

const deleteTemporaryReservation = (id: AdhocReservationId) => {
  return API.delete(`/adhoc-reservation/${id}`);
};

const BookingService = {
  calendar,
  check,
  submitBooking,
  get,
  cancel,
  amend,
  openDoor,
  getAudioRecording,
  initiateTossPayment,
  confirmTossPayment,
  cancelTossPayment,
  getTossPaymentState,
  deleteTemporaryReservation,
};

export default BookingService;
