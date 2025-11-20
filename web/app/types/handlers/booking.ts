import type { DateTime, IdentityId, UnitId } from "../models/base";
import type {
  Booking,
  CashPaymentStatus,
  OccupiedSlot,
} from "../models/booking";

export interface CalendarResponse {
  start: DateTime;
  end: DateTime;
  maxBookingHours: number;
  slots: OccupiedSlot[];
}

export interface CheckRequest {
  unitId: UnitId;
  timeFrom: DateTime;
  desiredHours: number;
}

export interface CheckResponse {
  totalPrice: number;
}

export interface SubmitBookingRequest {
  unitId: UnitId;
  timeFrom: DateTime;
  desiredHours: number;
  identityId: IdentityId;
  depositorName: string;
}

export interface SubmitBookingResponse {
  booking: Booking;
  cashPaymentStatus: CashPaymentStatus;
}

export interface GetBookingResponse {
  booking: Booking;
  cashPaymentStatus: CashPaymentStatus;
}

export interface CancelBookingResponse {
  cashPaymentStatus: CashPaymentStatus | null;
}

export interface AmendBookingRequest {
  newIdentityId: IdentityId | null;
}

export interface AmendBookingResponse {
  booking: Booking;
}
