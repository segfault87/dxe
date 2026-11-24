import type { AdHocPaymentId, DateTime } from "./base";
import type { Identity } from "./identity";

export interface CashTransaction {
  depositorName: string;
  price: number;
  confirmedAt: DateTime | null;
  refundPrice: number | null;
  refundAccount: string | null;
  refundedAt: DateTime | null;
  isRefundRequested: boolean;
  isRefunded: boolean;
}

export interface TossPaymentsTransaction {
  price: number;
  confirmedAt: DateTime | null;
  refundPrice: number | null;
  refundedAt: DateTime | null;
  isRefunded: boolean;
}

export interface Transaction {
  cash?: CashTransaction;
  tossPayments?: TossPaymentsTransaction;
}

export type AdHocPaymentStatus = "PENDING" | "PAID" | "CANCELED";

export interface AdHocPayment {
  id: AdHocPaymentId;
  recipient: Identity;
  price: number;
  note: string;
  status: AdHocPaymentStatus;
  createdAt: DateTime;
  paidAt: DateTime | null;
  canceledAt: DateTime | null;
}

export interface AdHocPaymentWithUrl extends AdHocPayment {
  url: string;
}
