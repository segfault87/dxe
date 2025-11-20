import type { DateTime } from "./base";

export type UserId = string;

export interface SelfUser {
  id: UserId;
  name: string;
  licensePlateNumber?: string;
  createdAt: DateTime;
  isAdministrator: boolean;
}

export interface User {
  id: UserId;
  name: string;
  createdAt: DateTime;
}
