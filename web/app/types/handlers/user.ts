import type { UserId, UnitId } from "../models/base";
import type { Booking } from "../models/booking";
import type { Group, GroupWithUsers } from "../models/group";
import type { SelfUser } from "../models/user";

export interface GetGroupResponse {
  group: GroupWithUsers;
}

export interface AmendGroupRequest {
  newName?: string;
  newOwner?: UserId;
  isOpen?: boolean;
}

export interface AmendGroupResponse {
  group: Group;
}

export interface CreateGroupRequest {
  name: string;
}

export interface CreateGroupResponse {
  group: GroupWithUsers;
}

export interface ListGroupsResponse {
  groups: GroupWithUsers[];
}

export interface MeResponse {
  user: SelfUser;
  activeBookings: Record<UnitId, Booking>;
  pendingBookings: Record<UnitId, Booking[]>;
}

export interface UpdateMeRequest {
  newName?: string;
  newLicensePlateNumber?: string;
}

export interface UpdateMeResponse {
  user: SelfUser;
}
