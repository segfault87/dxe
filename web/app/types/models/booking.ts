import type { BookingId, DateTime, AdhocReservationId, UnitId } from "./base";
import type { Identity } from "./identity";
import type { User } from "./user";

export type BookingStatus =
  | "PENDING"
  | "CONFIRMED"
  | "OVERDUE"
  | "CANCELED"
  | "BUFFERED"
  | "IN_PROGRESS"
  | "COMPLETE";

export interface Booking {
  id: BookingId;
  unitId: UnitId;
  holder: User;
  customer: Identity;
  bookingStart: DateTime;
  bookingEnd: DateTime;
  bookingHours: number;
  createdAt: DateTime;
  confirmedAt: DateTime | null;
  isConfirmed: boolean;
  canceledAt: DateTime | null;
  isCanceled: boolean;
  status: BookingStatus;
}

export interface CashPaymentStatus {
  depositorName: string;
  price: number;
  confirmedAt: DateTime | null;
  refundPrice: number | null;
  refundAccount: string | null;
  refundedAt: DateTime | null;
  isRefundRequested: boolean;
  isRefunded: boolean;
}

export interface BookingWithPayments {
  booking: Booking;
  payment: CashPaymentStatus | null;
}

export interface OccupiedSlot {
  maskedName: string;
  bookingDate: DateTime;
  bookingHours: number;
  confirmed: boolean;
}

export interface AdhocReservation {
  id: AdhocReservationId;
  holder: User;
  reservationStart: DateTime;
  reservationEnd: DateTime;
  reservedHours: number;
  temporary: boolean;
  remark: string | null;
}

export interface AudioRecording {
  bookingId: BookingId;
  url: string;
  createdAt: DateTime;
  expiresIn: DateTime | null;
}
