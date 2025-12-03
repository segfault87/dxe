import type { DateTime, IdentityId, SpaceId, UnitId } from "../models/base";
import type {
  AdhocParking,
  AdhocReservation,
  Booking,
  BookingWithPayments,
  CashPaymentStatus,
} from "../models/booking";
import type { GroupWithUsers } from "../models/group";
import type { SelfUser } from "../models/user";

export interface GetBookingsResponse {
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

export interface GetAdhocReservationsResponse {
  reservations: AdhocReservation[];
}

export interface CreateAdhocReservationRequest {
  unitId: UnitId;
  customerId: IdentityId;
  timeFrom: DateTime;
  desiredHours: number;
  temporary: boolean;
  remark: string | null;
}

export interface CreateAdhocReservationResponse {
  reservation: AdhocReservation;
}

export interface GetGroupsResponse {
  groups: GroupWithUsers[];
}

export interface GetUsersResponse {
  users: SelfUser[];
}

export interface GetAdhocParkingsResponse {
  parkings: AdhocParking[];
}

export interface CreateAdhocParkingRequest {
  spaceId: SpaceId;
  timeFrom: DateTime;
  desiredHours: number;
  licensePlateNumber: string;
}
