import type {
  AdhocReservationId,
  BookingId,
  DateTime,
  ForeignPaymentId,
  IdentityId,
  UnitId,
} from "../models/base";
import type {
  AudioRecording,
  Booking,
  CashPaymentStatus,
  OccupiedSlot,
  TossPaymentStatus,
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
  cashPaymentStatus: CashPaymentStatus | null;
  tossPaymentStatus: TossPaymentStatus | null;
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

export interface GetAudioRecordingResponse {
  audioRecording: AudioRecording | null;
}

export interface TossPaymentInitiateRequest {
  temporaryReservationId: AdhocReservationId | null;
  unitId: UnitId;
  timeFrom: DateTime;
  desiredHours: number;
  identityId: IdentityId;
}

export interface TossPaymentInitiateResponse {
  orderId: ForeignPaymentId;
  price: number;
  temporaryReservationId: AdhocReservationId;
  expiresIn: DateTime;
}

export interface TossPaymentConfirmRequest {
  paymentKey: string;
  orderId: ForeignPaymentId;
  amount: number;
}

export interface TossPaymentConfirmResponse {
  bookingId: BookingId;
}

export interface GetTossPaymentStateResponse {
  timeFrom: DateTime;
  timeTo: DateTime;
}
