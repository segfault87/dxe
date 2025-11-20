import Modal from "./Modal";
import type { ModalProps } from "./Modal";
import { useEnv } from "../context/EnvContext";
import type { GroupWithUsers } from "../types/models/group";

export interface GroupInvitationProps extends ModalProps {
  group: GroupWithUsers;
}

export default function GroupInvitation(props: GroupInvitationProps) {
  const env = useEnv();
  const group = props.group;

  const invitationUrl = `${env.urlBase}/api/join/${group.id}`;

  const copy = async () => {
    await navigator.clipboard.writeText(invitationUrl);
  };

  return (
    <Modal {...props}>
      <div>
        <p>
          <em>{group.name}</em> 그룹에 구성원을 추가하려면 아래 링크를
          구성원들에게 공유해주시기 바랍니다.
        </p>
        <input
          style={{ width: "240px" }}
          type="text"
          value={invitationUrl}
          readOnly={true}
          onClick={(e) => {
            (e.target as HTMLInputElement).select();
          }}
        />
        <button
          style={{ marginLeft: "16px" }}
          className="primary"
          onClick={copy}
        >
          복사
        </button>
        <h4>구성원 목록</h4>
        <ul>
          {props.group.users.map((e) => (
            <li key={e.id}>
              {e.name} ({new Date(e.createdAt).toLocaleString()}에 추가)
            </li>
          ))}
        </ul>
        <button
          style={{ margin: "0", marginTop: "16px", width: "100%" }}
          className="primary"
          onClick={props.close}
        >
          닫기
        </button>
      </div>
    </Modal>
  );
}
