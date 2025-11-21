import API from "../api";
import type {
  GetPendingBookingsResponse,
  GetReservationsResponse,
  ModifyBookingResponse,
  ModifyBookingRequest,
  CreateReservationRequest,
  CreateReservationResponse,
  GetRefundPendingBookingsResponse,
  GetUsersResponse,
  GetGroupsResponse,
} from "../types/handlers/admin";
import type { BookingId, ReservationId, UnitId } from "../types/models/base";

const getPendingBookings = () => {
  return API.get<GetPendingBookingsResponse>("/admin/bookings/pending");
};

const getRefundPendingBookings = () => {
  return API.get<GetRefundPendingBookingsResponse>(
    "/admin/bookings/pending-refunds",
  );
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
  getPendingBookings,
  getRefundPendingBookings,
  getUsers,
  getGroups,
  modifyBooking,
  getReservations,
  createReservation,
  deleteReservation,
};

export default AdminService;
