import { useState } from "react";

import Modal from "./Modal";
import type { ModalProps } from "./Modal";
import UserService from "../api/user";
import { defaultErrorHandler } from "../lib/error";
import type { GroupWithUsers } from "../types/models/group";

export interface GroupManagementProps extends ModalProps {
  group: GroupWithUsers;
  onUpdate: () => void;
}

export default function GroupManagement(props: GroupManagementProps) {
  const [name, setName] = useState(props.group.name);
  const [isOpen, setOpen] = useState(props.group.isOpen);

  const [isRequestInProgress, setRequestInProgress] = useState(false);

  const hasChanged = name !== props.group.name || isOpen !== props.group.isOpen;

  const update = async () => {
    setRequestInProgress(true);
    try {
      await UserService.amendGroup(props.group.id, {
        newName: name,
        isOpen: isOpen,
      });
      props.onUpdate();
      props.close();
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  const deleteGroup = async () => {
    if (!confirm(`${props.group.name} 그룹을 정말 삭제하시겠습니까?`)) {
      return;
    }

    setRequestInProgress(true);

    try {
      await UserService.deleteGroup(props.group.id);
      props.onUpdate();
      props.close();
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  return (
    <Modal {...props}>
      <label htmlFor="name">그룹명</label>
      <br />
      <input
        type="text"
        value={name}
        disabled={isRequestInProgress}
        onChange={(e) => setName(e.target.value)}
      />
      <div style={{ marginTop: "16px" }}>
        <input
          type="checkbox"
          id="is-open"
          checked={isOpen}
          onChange={(e) => setOpen(e.target.checked)}
        />{" "}
        <label htmlFor="is-open">멤버 추가 허용</label>
      </div>
      <div style={{ marginTop: "16px" }}>
        {props.group.users.length > 1 ? (
          <>그룹을 삭제하려면 다른 구성원이 속해있지 않아야 합니다.</>
        ) : null}
      </div>
      <button
        className="primary"
        disabled={props.group.users.length > 1 || isRequestInProgress}
        onClick={deleteGroup}
      >
        그룹 삭제
      </button>
      <div className="actions">
        <button
          className="primary"
          onClick={update}
          disabled={!hasChanged || isRequestInProgress}
        >
          저장
        </button>
        <button
          className="primary"
          onClick={() => {
            props.close();
          }}
        >
          닫기
        </button>
      </div>
    </Modal>
  );
}
