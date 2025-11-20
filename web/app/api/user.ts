import API from "../api";
import type { GroupId } from "../types/models/base";
import type {
  AmendGroupRequest,
  AmendGroupResponse,
  CreateGroupRequest,
  CreateGroupResponse,
  GetGroupResponse,
  ListGroupsResponse,
  MeResponse,
  UpdateMeRequest,
  UpdateMeResponse,
} from "../types/handlers/user";

const me = () => {
  return API.get<MeResponse>("/user/me");
};

const updateMe = (data: UpdateMeRequest) => {
  return API.post<UpdateMeResponse>("/user/me", data);
};

const getGroup = (groupId: GroupId) => {
  return API.get<GetGroupResponse>(`/user/group/${groupId}`);
};

const amendGroup = (groupId: GroupId, data: AmendGroupRequest) => {
  return API.put<AmendGroupResponse>(`/user/group/${groupId}`, data);
};

const deleteGroup = (groupId: GroupId) => {
  return API.delete(`/user/group/${groupId}`);
};

const listGroups = () => {
  return API.get<ListGroupsResponse>("/user/groups");
};

const createGroup = (data: CreateGroupRequest) => {
  return API.post<CreateGroupResponse>("/user/groups", data);
};

const joinGroup = (groupId: GroupId) => {
  return API.put(`/user/group/${groupId}/membership`);
};

const leaveGroup = (groupId: GroupId) => {
  return API.delete(`/user/group/${groupId}/membersip`);
};

const UserService = {
  me,
  updateMe,
  getGroup,
  amendGroup,
  deleteGroup,
  listGroups,
  createGroup,
  joinGroup,
  leaveGroup,
};

export default UserService;
