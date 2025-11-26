import API from "../api";
import { toUtcIso8601 } from "../lib/datetime";
import type {
  GetBookingsResponse,
  GetReservationsResponse,
  ModifyBookingResponse,
  ModifyBookingRequest,
  CreateReservationRequest,
  CreateReservationResponse,
  GetUsersResponse,
  GetGroupsResponse,
} from "../types/handlers/admin";
import type { BookingId, ReservationId, UnitId } from "../types/models/base";

const getBookings = (
  type: "confirmed" | "pending" | "refund_pending" | "canceled",
  dateFrom?: Date,
) => {
  let query = `type=${type}`;
  if (dateFrom) {
    query += `&date_from=${toUtcIso8601(dateFrom)}`;
  }

  return API.get<GetBookingsResponse>(`/admin/bookings?${query}`);
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

const getReservations = (unitId: UnitId) => {
  return API.get<GetReservationsResponse>(
    `/admin/reservations?unit_id=${unitId}`,
  );
};

const createReservation = (data: CreateReservationRequest) => {
  return API.post<CreateReservationResponse>("/admin/reservations", data);
};

const deleteReservation = (id: ReservationId) => {
  return API.delete(`/admin/reservation/${id}`);
};

const AdminService = {
  getBookings,
  getUsers,
  getGroups,
  modifyBooking,
  getReservations,
  createReservation,
  deleteReservation,
};

export default AdminService;
