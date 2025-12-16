import type {
  AdhocReservationId,
  BookingId,
  DateTime,
  ForeignPaymentId,
  ProductType,
  IdentityId,
  UnitId,
} from "../models/base";
import type { AudioRecording, Booking, OccupiedSlot } from "../models/booking";
import type { CashTransaction, Transaction } from "../models/payment";

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
  additionalHours?: number;
  excludeBookingId?: BookingId;
  excludeAdhocReservationId?: AdhocReservationId;
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
  cashTransaction: CashTransaction;
}

export interface GetBookingResponse {
  booking: Booking;
  transaction: Transaction | null;
  amendable: boolean;
  extendableHours: number;
}

export interface CancelBookingResponse {
  transaction: Transaction | null;
}

export interface AmendBookingRequest {
  newIdentityId?: IdentityId;
  newTimeFrom?: DateTime;
  additionalHours?: number;
}

export interface AmendBookingResponse {
  booking: Booking;
  foreignPaymentId: ForeignPaymentId | null;
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
  type: ProductType;
  timeFrom: DateTime;
  timeTo: DateTime;
}
