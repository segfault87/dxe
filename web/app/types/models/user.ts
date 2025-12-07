import type { DateTime } from "./base";

export type UserId = string;

export interface SelfUser {
  id: UserId;
  name: string;
  licensePlateNumber?: string;
  createdAt: DateTime;

  isAdministrator: boolean;

  depositorName: string | null;
  refundAccount: string | null;
  usePgPayment: boolean;
}

export interface User {
  id: UserId;
  name: string;
  createdAt: DateTime;
}
