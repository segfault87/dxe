import type {
  AdhocParkingId,
  AdhocReservationId,
  BookingId,
  DateTime,
  UnitId,
} from "./base";
import type { Identity } from "./identity";
import type { Transaction } from "./payment";
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

export interface BookingWithPayments {
  booking: Booking;
  transaction: Transaction | null;
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
  deletedAt: DateTime;
  remark: string | null;
}

export interface AudioRecording {
  bookingId: BookingId;
  url: string;
  createdAt: DateTime;
  expiresIn: DateTime | null;
}

export interface AdhocParking {
  id: AdhocParkingId;
  unitId: UnitId;
  timeFrom: DateTime;
  timeTo: DateTime;
  licensePlateNumber: string;
  createdAt: DateTime;
}
