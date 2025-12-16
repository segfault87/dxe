import type { DateTime } from "./base";

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
