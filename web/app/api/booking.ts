import API from "../api";
import type {
  AmendBookingResponse,
  AmendBookingRequest,
  CalendarResponse,
  CancelBookingResponse,
  CheckRequest,
  CheckResponse,
  GetBookingResponse,
  SubmitBookingRequest,
  SubmitBookingResponse,
} from "../types/handlers/booking";
import type { BookingId, UnitId } from "../types/models/base";

const calendar = (unitId: UnitId) => {
  return API.get<CalendarResponse>(`/bookings/calendar?unit_id=${unitId}`);
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

const cancel = (bookingId: BookingId, refundAccount: string | null) => {
  if (refundAccount !== null) {
    return API.delete<CancelBookingResponse>(
      `/booking/${bookingId}?refund_account=${refundAccount}`,
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

const BookingService = {
  calendar,
  check,
  submitBooking,
  get,
  cancel,
  amend,
  openDoor,
};

export default BookingService;
