import { useEffect, useState } from "react";
import { Link } from "react-router";

import "./MyPage.css";
import type { Route } from "./+types/MyPage";
import UserService from "../api/user";
import GroupInvitation from "../components/GroupInvitation";
import GroupManagementModal from "../components/GroupManagement";
import checkPlateNumber from "../lib/PlateNumber";
import RequiresAuth from "../lib/RequiresAuth";
import { DEFAULT_UNIT_ID } from "../constants";
import { useAuth } from "../context/AuthContext";
import Section from "../components/Section";
import { defaultErrorHandler } from "../lib/error";
import type { GroupId } from "../types/models/base";
import type { Booking } from "../types/models/booking";
import type { GroupWithUsers } from "../types/models/group";
import type { SelfUser } from "../types/models/user";

export function meta({}: Route.MetaArgs) {
  return [{ title: "예약 및 설정 | 드림하우스 합주실" }];
}

function ActiveBookingSection({ booking }: { booking: Booking }) {
  const bookingStart = new Date(booking.bookingStart);
  const bookingEnd = new Date(booking.bookingEnd);

  return (
    <div>
      <Link to={`/reservation/${booking.id}`}>
        {bookingStart.toLocaleString()} - {bookingEnd.toLocaleString()} |{" "}
        {booking.customer.name}
      </Link>
    </div>
  );
}

function PendingBookingsSection({ bookings }: { bookings: Booking[] }) {
  if (bookings.length === 0) {
    return <div>다음 예약이 없습니다.</div>;
  } else {
    const items = bookings.map((booking) => {
      const bookingStart = new Date(booking.bookingStart);
      const bookingEnd = new Date(booking.bookingEnd);
      return (
        <li key={booking.id}>
          <Link to={`/reservation/${booking.id}`}>
            {bookingStart.toLocaleString()} - {bookingEnd.toLocaleString()} |{" "}
            {booking.customer.name}
            {!booking.isConfirmed ? " (미확정)" : null}
          </Link>
        </li>
      );
    });

    return <ul>{items}</ul>;
  }
}

function GroupManagement({ me }: { me: SelfUser }) {
  const [groups, setGroups] = useState<GroupWithUsers[]>([]);
  const [newGroupName, setNewGroupName] = useState("");

  const [isRequestInProgress, setRequestInProgress] = useState(false);

  const [inviteGroupModal, setInviteGroupModal] =
    useState<GroupWithUsers | null>(null);
  const [groupManagementModal, setGroupManagementModal] =
    useState<GroupWithUsers | null>(null);

  const createGroup = async (name: string) => {
    try {
      setRequestInProgress(true);
      const result = await UserService.createGroup({ name });
      setGroups([...groups, result.data.group]);
      setNewGroupName("");
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  const leaveGroup = async (groupId: GroupId) => {
    if (!confirm("그룹을 나가시겠습니까?")) {
      return;
    }

    try {
      setRequestInProgress(true);
      await UserService.leaveGroup(groupId);
      await fetch();
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  const fetch = async () => {
    try {
      const groups = await UserService.listGroups();
      setGroups(groups.data.groups);
    } catch (error) {
      defaultErrorHandler(error);
    }
  };

  useEffect(() => {
    fetch();
  }, []);

  let list = null;
  if (groups.length === 0) {
    list = <div>현재 속해있는 그룹이 없습니다.</div>;
  } else {
    const items = groups.map((e) => (
      <li key={e.id}>
        {e.name} (총 {e.users.length}명)
        <br />
        {e.isOpen ? (
          <a
            onClick={() => {
              setInviteGroupModal(e);
            }}
          >
            멤버 초대
          </a>
        ) : null}{" "}
        {e.ownerId === me.id ? (
          <a
            onClick={() => {
              setGroupManagementModal(e);
            }}
          >
            그룹 설정
          </a>
        ) : null}{" "}
        {e.ownerId !== me.id ? (
          <a onClick={() => leaveGroup(e.id)}>그룹 나가기</a>
        ) : null}
      </li>
    ));

    list = <ul>{items}</ul>;
  }

  return (
    <>
      <div className="group-management">
        {list}
        <hr />
        <h4>새 그룹 만들기</h4>
        <input
          type="text"
          className="field-single-row"
          value={newGroupName}
          placeholder="그룹명"
          disabled={isRequestInProgress}
          maxLength={40}
          onChange={(e) => setNewGroupName(e.target.value)}
        />
        <button
          className="primary"
          onClick={() => {
            createGroup(newGroupName);
          }}
          disabled={newGroupName.length === 0 || isRequestInProgress}
        >
          추가
        </button>
      </div>
      {inviteGroupModal !== null ? (
        <GroupInvitation
          isOpen={inviteGroupModal !== null}
          close={() => {
            setInviteGroupModal(null);
          }}
          group={inviteGroupModal}
        />
      ) : null}
      {groupManagementModal !== null ? (
        <GroupManagementModal
          isOpen={groupManagementModal !== null}
          close={() => {
            setGroupManagementModal(null);
          }}
          group={groupManagementModal}
          onUpdate={() => {
            fetch();
          }}
        />
      ) : null}
    </>
  );
}

function UserProfile({ me }: { me: SelfUser }) {
  const [name, setName] = useState(me.name);
  const [licensePlateNumber, setLicensePlateNumber] = useState(
    me.licensePlateNumber ?? "",
  );

  const [isRequestInProgress, setRequestInProgress] = useState(false);

  const changeName = async (name: string) => {
    setRequestInProgress(true);
    try {
      const result = await UserService.updateMe({ newName: name });
      setName(result.data.user.name);
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  const changeLicensePlateNumber = async (licensePlateNumber: string) => {
    setRequestInProgress(true);
    try {
      const result = await UserService.updateMe({
        newLicensePlateNumber: licensePlateNumber,
      });
      setLicensePlateNumber(result.data.user.licensePlateNumber ?? "");
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  return (
    <div>
      <section>
        <label htmlFor="name">성함 (닉네임)</label>
        <br />
        <input
          className="field-single-row"
          type="text"
          name="name"
          value={name}
          disabled={isRequestInProgress}
          onChange={(e) => setName(e.target.value)}
        />
        <button
          className="primary"
          onClick={() => {
            changeName(name);
          }}
          disabled={
            name.length === 0 || isRequestInProgress || me.name === name
          }
        >
          변경
        </button>
      </section>
      <section>
        <label htmlFor="license-plate-number">차량번호</label>
        <br />
        <input
          className="field-single-row"
          type="text"
          name="license-plate-number"
          value={licensePlateNumber}
          disabled={isRequestInProgress}
          onChange={(e) => setLicensePlateNumber(e.target.value)}
        />
        <button
          className="primary"
          onClick={() => {
            changeLicensePlateNumber(licensePlateNumber);
          }}
          disabled={
            checkPlateNumber(licensePlateNumber) === false ||
            isRequestInProgress ||
            me.licensePlateNumber === licensePlateNumber
          }
        >
          변경
        </button>
        <hr />
        <a className="primary" href="/api/auth/logout">
          로그아웃
        </a>
      </section>
    </div>
  );
}

function MyPage() {
  const auth = useAuth();

  if (!auth) {
    return;
  }

  return (
    <>
      {auth.activeBookings[DEFAULT_UNIT_ID] ? (
        <Section id="active-bookings" title="현재 예약">
          <ActiveBookingSection
            booking={auth.activeBookings[DEFAULT_UNIT_ID]}
          />
        </Section>
      ) : null}
      <Section id="pending-bookings" title="예약 현황">
        <PendingBookingsSection
          bookings={auth.pendingBookings[DEFAULT_UNIT_ID] ?? []}
        />
      </Section>
      <Section id="group-management" title="그룹 관리">
        <GroupManagement me={auth.user} />
      </Section>
      <Section id="user-info" title="사용자 정보 변경">
        <UserProfile me={auth.user} />
      </Section>
    </>
  );
}

export default RequiresAuth(MyPage);
