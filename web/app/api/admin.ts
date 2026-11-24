import API from "../api";
import { toUtcIso8601 } from "../lib/datetime";
import type {
  CancelBookingResponse,
  CreateAdHocPaymentRequest,
  CreateAdHocPaymentResponse,
  GetAdHocPaymentsResponse,
  CreateBookingRequest,
  CreateBookingResponse,
  CreateAdhocParkingRequest,
  CreateAdhocReservationRequest,
  CreateAdhocReservationResponse,
  GetAdhocParkingsResponse,
  GetAdhocReservationsResponse,
  GetBookingResponse,
  GetBookingsResponse,
  ModifyBookingResponse,
  ModifyBookingRequest,
  GetUsersResponse,
  GetGroupsResponse,
} from "../types/handlers/admin";
import type {
  AdHocPaymentId,
  AdhocParkingId,
  AdhocReservationId,
  BookingId,
  SpaceId,
  UnitId,
} from "../types/models/base";

const getBooking = (bookingId: BookingId) => {
  return API.get<GetBookingResponse>(`/admin/booking/${bookingId}`);
};

const getBookings = (
  type: "complete" | "confirmed" | "pending" | "refund_pending" | "canceled",
  dateFrom?: Date,
  dateTo?: Date,
  limit?: number,
  offset?: number,
) => {
  let query = `type=${type}`;
  if (dateFrom) {
    query += `&date_from=${encodeURIComponent(toUtcIso8601(dateFrom))}`;
  }
  if (dateTo) {
    query += `&date_to=${encodeURIComponent(toUtcIso8601(dateTo))}`;
  }
  if (limit) {
    query += `&limit=${limit}`;
  }
  if (offset) {
    query += `&offset=${offset}`;
  }

  return API.get<GetBookingsResponse>(`/admin/bookings?${query}`);
};

const createBooking = (data: CreateBookingRequest) => {
  return API.post<CreateBookingResponse>("/admin/bookings", data);
};

const getUsers = () => {
  return API.get<GetUsersResponse>("/admin/users");
};

const getGroups = () => {
  return API.get<GetGroupsResponse>("/admin/groups");
};

const modifyBooking = (bookingId: BookingId, data: ModifyBookingRequest) => {
  return API.put<ModifyBookingResponse>(`/admin/booking/${bookingId}`, data);
};

const getAdhocReservations = (unitId: UnitId) => {
  return API.get<GetAdhocReservationsResponse>(
    `/admin/adhoc-reservations?unit_id=${unitId}`,
  );
};

const createAdhocReservation = (data: CreateAdhocReservationRequest) => {
  return API.post<CreateAdhocReservationResponse>(
    "/admin/adhoc-reservations",
    data,
  );
};

const deleteAdhocReservation = (id: AdhocReservationId) => {
  return API.delete(`/admin/adhoc-reservation/${id}`);
};

const getAdhocParkings = (spaceId: SpaceId) => {
  return API.get<GetAdhocParkingsResponse>(
    `/admin/adhoc-parkings?space_id=${spaceId}`,
  );
};

const createAdhocParking = (data: CreateAdhocParkingRequest) => {
  return API.post("/admin/adhoc-parkings", data);
};

const deleteAdhocParking = (id: AdhocParkingId) => {
  return API.delete(`/admin/adhoc-parking/${id}`);
};

const cancelBooking = (id: BookingId) => {
  return API.delete<CancelBookingResponse>(`/admin/booking/${id}`);
}

const getAdHocPayments = () => {
  return API.get<GetAdHocPaymentsResponse>("/admin/adhoc-payments");
};

const createAdHocPayment = (data: CreateAdHocPaymentRequest) => {
  return API.post<CreateAdHocPaymentResponse>("/admin/adhoc-payments", data);
};

const cancelAdHocPayment = (id: AdHocPaymentId) => {
  return API.delete(`/admin/adhoc-payment/${id}`);
};

const AdminService = {
  cancelBooking,
  getAdHocPayments,
  createAdHocPayment,
  cancelAdHocPayment,
  createBooking,
  getBooking,
  getBookings,
  getUsers,
  getGroups,
  modifyBooking,
  getAdhocReservations,
  createAdhocReservation,
  deleteAdhocReservation,
  getAdhocParkings,
  createAdhocParking,
  deleteAdhocParking,
};

export default AdminService;
