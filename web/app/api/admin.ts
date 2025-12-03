import API from "../api";
import { toUtcIso8601 } from "../lib/datetime";
import type {
  CreateAdhocParkingRequest,
  CreateAdhocReservationRequest,
  CreateAdhocReservationResponse,
  GetAdhocParkingsResponse,
  GetAdhocReservationsResponse,
  GetBookingsResponse,
  ModifyBookingResponse,
  ModifyBookingRequest,
  GetUsersResponse,
  GetGroupsResponse,
} from "../types/handlers/admin";
import type {
  AdhocParkingId,
  AdhocReservationId,
  BookingId,
  SpaceId,
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

const AdminService = {
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
