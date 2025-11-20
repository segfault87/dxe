import { useEffect, useState } from "react";
import { Link } from "react-router";

import "./GroupSelection.css";
import Modal from "./Modal";
import type { ModalProps } from "./Modal";
import UserService from "../api/user";
import { defaultErrorHandler } from "../lib/error";
import type { GroupWithUsers } from "../types/models/group";
import type { BookingId, GroupId } from "../types/models/base";

export interface GroupSelectionProps extends ModalProps {
  bookingId: BookingId;
  onSelectGroup: (group: GroupWithUsers) => void;
  title: string;
}

export default function GroupSelection(props: GroupSelectionProps) {
  const [groups, setGroups] = useState<GroupWithUsers[] | null>(null);

  const [selectedGroupId, setSelectedGroupId] = useState<GroupId | null>(null);

  useEffect(() => {
    const getGroups = async () => {
      try {
        const result = await UserService.listGroups();
        setGroups(result.data.groups);
      } catch (error) {
        defaultErrorHandler(error);
      }
    };

    getGroups();
  }, []);

  return (
    <Modal {...props}>
      {groups === null ? null : groups.length === 0 ? (
        <div className="no-contents">
          <p>
            현재 소속중인 그룹이 없습니다.
            <br />
            그룹은 <Link to="/my">설정</Link> 페이지에서 만들 수 있습니다.
          </p>
        </div>
      ) : (
        <div className="contents">
          <h3>{props.title}</h3>
          <ul>
            {groups.map((group) => (
              <li key={group.id}>
                <input
                  type="radio"
                  name="groupId"
                  checked={selectedGroupId === group.id}
                  value={group.id}
                  onChange={(e) => setSelectedGroupId(e.target.value)}
                  id={`group-${group.id}`}
                />
                <label htmlFor={`group-${group.id}`}>{group.name}</label>
              </li>
            ))}
          </ul>
        </div>
      )}
      <div className="actions">
        <button
          className="primary"
          onClick={() => {
            if (selectedGroupId !== null) {
              const group = groups?.find(
                (group) => group.id === selectedGroupId,
              );
              if (group !== undefined) {
                props.onSelectGroup(group);
              }
            }
          }}
          disabled={selectedGroupId === null}
        >
          확인
        </button>
        <button
          className="primary"
          onClick={() => {
            props.close();
          }}
        >
          취소
        </button>
      </div>
    </Modal>
  );
}
