import type { DateTime, UnitId } from "../models/base";
import type {
  Booking,
  BookingWithPayments,
  CashPaymentStatus,
  Reservation,
} from "../models/booking";
import type { GroupWithUsers } from "../models/group";
import type { SelfUser } from "../models/user";

export interface GetPendingBookingsResponse {
  bookings: BookingWithPayments[];
}

export interface GetRefundPendingBookingsResponse {
  bookings: BookingWithPayments[];
}

export type BookingAction = "CONFIRM" | "REFUND" | "CANCEL";

export interface ModifyBookingRequest {
  action: BookingAction;
}

export interface ModifyBookingResponse {
  booking: Booking;
  cashPaymentStatus: CashPaymentStatus;
}

export interface GetReservationsResponse {
  reservations: Reservation[];
}

export interface CreateReservationRequest {
  unitId: UnitId;
  timeFrom: DateTime;
  desiredHours: number;
  temporary: boolean;
  remark: string | null;
}

export interface CreateReservationResponse {
  reservation: Reservation;
}

export interface GetGroupsResponse {
  groups: GroupWithUsers[];
}

export interface GetUsersResponse {
  users: SelfUser[];
}
