import API from "../api";
import { toUtcIso8601 } from "../lib/datetime";
import type {
  CreateAdhocReservationRequest,
  CreateAdhocReservationResponse,
  GetAdhocReservationsResponse,
  GetBookingsResponse,
  ModifyBookingResponse,
  ModifyBookingRequest,
  GetUsersResponse,
  GetGroupsResponse,
} from "../types/handlers/admin";
import type {
  BookingId,
  AdhocReservationId,
  UnitId,
} from "../types/models/base";

const getBookings = (
  type: "confirmed" | "pending" | "refund_pending" | "canceled",
  dateFrom?: Date,
) => {
  let query = `type=${type}`;
  if (dateFrom) {
    query += `&date_from=${encodeURIComponent(toUtcIso8601(dateFrom))}`;
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

const AdminService = {
  getBookings,
  getUsers,
  getGroups,
  modifyBooking,
  getAdhocReservations,
  createAdhocReservation,
  deleteAdhocReservation,
};

export default AdminService;
