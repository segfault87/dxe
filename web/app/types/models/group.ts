import type { DateTime, GroupId, UserId } from "./base";
import type { User } from "./user";

export interface Group {
  id: GroupId;
  name: string;
  ownerId: UserId;
  isOpen: boolean;
  createdAt: DateTime;
}

export interface GroupWithUsers extends Group {
  users: User[];
}
