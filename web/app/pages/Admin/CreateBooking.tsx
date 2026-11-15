import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router";

import type { Route } from "./+types/CreateBooking";
import AdminService from "../../api/admin";
import BookingService from "../../api/booking";
import AttachmentSelection from "../../components/AttachmentSelection";
import { DEFAULT_UNIT_ID } from "../../constants";
import { toUtcIso8601 } from "../../lib/datetime";
import { defaultErrorHandler, loaderErrorHandler } from "../../lib/error";
import type { BookingAttachmentId } from "../../types/models/base";
import type { BookingAttachment } from "../../types/models/booking";
import type { GroupWithUsers } from "../../types/models/group";
import type { SelfUser } from "../../types/models/user";

interface LoaderData {
  users: SelfUser[];
  groups: GroupWithUsers[];
  attachments: BookingAttachment[];
}

export async function clientLoader({
  request,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  try {
    const [users, groups, attachments] = await Promise.all([
      AdminService.getUsers(),
      AdminService.getGroups(),
      BookingService.getAttachments(),
    ]);

    return {
      users: users.data.users,
      groups: groups.data.groups,
      attachments: attachments.data.attachments,
    };
  } catch (error) {
    throw loaderErrorHandler(error, request.url);
  }
}

// Value for <input type="datetime-local">, in local time
function toLocalInputValue(date: Date): string {
  const pad = (v: number) => v.toString().padStart(2, "0");

  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:00`;
}

export default function CreateBooking({ loaderData }: Route.ComponentProps) {
  const { users, groups, attachments } = loaderData;
  const navigate = useNavigate();

  const [holderId, setHolderId] = useState("");
  const [customerId, setCustomerId] = useState("");
  const [startTime, setStartTime] = useState(() => {
    const date = new Date();
    date.setHours(date.getHours() + 1, 0, 0, 0);
    return toLocalInputValue(date);
  });
  const [desiredHours, setDesiredHours] = useState("2");
  const [selectedAttachmentIds, setSelectedAttachmentIds] = useState<
    BookingAttachmentId[]
  >([]);
  const [unavailableAttachmentIds, setUnavailableAttachmentIds] = useState<
    BookingAttachmentId[]
  >([]);
  const [skipPayment, setSkipPayment] = useState(true);
  const [depositorName, setDepositorName] = useState("");
  const [price, setPrice] = useState("");
  const [isSubmitting, setSubmitting] = useState(false);

  const sortedUsers = useMemo(
    () => [...users].sort((a, b) => a.name.localeCompare(b.name)),
    [users],
  );

  // Groups the holder belongs to come first, as those are the usual customers
  const sortedGroups = useMemo(() => {
    const isMember = (g: GroupWithUsers) =>
      g.users.some((u) => u.id === holderId);

    return [...groups].sort((a, b) => {
      const diff = Number(isMember(b)) - Number(isMember(a));
      return diff !== 0 ? diff : a.name.localeCompare(b.name);
    });
  }, [groups, holderId]);

  const timeFrom = useMemo(() => {
    const date = new Date(startTime);
    return isNaN(date.getTime()) ? null : date;
  }, [startTime]);
  const hours = parseInt(desiredHours);

  useEffect(() => {
    if (!timeFrom || !(hours > 0)) {
      setUnavailableAttachmentIds([]);
      return;
    }

    let canceled = false;
    BookingService.getAttachments(timeFrom, hours)
      .then((result) => {
        if (!canceled) {
          setUnavailableAttachmentIds(result.data.unavailableAttachmentIds);
        }
      })
      .catch(() => {});

    return () => {
      canceled = true;
    };
  }, [timeFrom, hours]);

  const onChangeHolder = (newHolderId: string) => {
    // Keep the customer in sync with the holder unless a group was chosen
    if (customerId === "" || customerId === holderId) {
      setCustomerId(newHolderId);
    }
    setHolderId(newHolderId);
  };

  const submit = async () => {
    if (!holderId || !customerId) {
      alert("예약자와 고객을 선택해 주세요.");
      return;
    }
    if (!timeFrom) {
      alert("시작시간을 입력해 주세요.");
      return;
    }
    if (!(hours > 0)) {
      alert("이용시간을 확인해 주세요.");
      return;
    }

    const attachmentIds = selectedAttachmentIds.filter(
      (v) => !unavailableAttachmentIds.includes(v),
    );

    let parsedPrice: number | null = null;
    if (!skipPayment && price.trim().length > 0) {
      parsedPrice = parseInt(price);
      if (isNaN(parsedPrice) || parsedPrice < 0) {
        alert("금액을 확인해 주세요.");
        return;
      }
    }

    const holder = users.find((v) => v.id === holderId);
    const customer =
      users.find((v) => v.id === customerId) ??
      groups.find((v) => v.id === customerId);

    const message = [
      `예약자: ${holder?.name ?? holderId}`,
      `고객: ${customer?.name ?? customerId}`,
      `시간: ${timeFrom.toLocaleString()} (${hours}시간)`,
      skipPayment
        ? "결제 없이 바로 확정되며, 고객에게 알림톡이 발송됩니다."
        : "입금 대기 상태로 생성됩니다.",
      "",
      "예약을 생성하시겠습니까?",
    ].join("\n");

    if (!confirm(message)) {
      return;
    }

    setSubmitting(true);
    try {
      const result = await AdminService.createBooking({
        unitId: DEFAULT_UNIT_ID,
        holderId,
        customerId,
        timeFrom: toUtcIso8601(timeFrom),
        desiredHours: hours,
        attachmentIds,
        skipPayment,
        depositorName: skipPayment ? null : depositorName.trim() || null,
        price: parsedPrice,
      });

      navigate(`/admin/booking/${result.data.booking.id}`);
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <>
      <h2>예약 생성</h2>
      <p>
        관리자 권한으로 예약을 생성합니다. 최대 이용시간 제한이 적용되지
        않습니다.
      </p>
      예약자:{" "}
      <select value={holderId} onChange={(e) => onChangeHolder(e.target.value)}>
        <option value="">선택</option>
        {sortedUsers.map((u) => (
          <option key={u.id} value={u.id}>
            {u.name} ({u.id})
          </option>
        ))}
      </select>
      <br />
      고객:{" "}
      <select value={customerId} onChange={(e) => setCustomerId(e.target.value)}>
        <option value="">선택</option>
        <optgroup label="개인">
          {sortedUsers.map((u) => (
            <option key={u.id} value={u.id}>
              {u.name}
              {u.id === holderId ? " (예약자 본인)" : ""}
            </option>
          ))}
        </optgroup>
        <optgroup label="그룹">
          {sortedGroups.map((g) => (
            <option key={g.id} value={g.id}>
              {g.name}
              {g.users.some((u) => u.id === holderId) ? " (예약자 소속)" : ""}
            </option>
          ))}
        </optgroup>
      </select>
      <br />
      시작시간:{" "}
      <input
        type="datetime-local"
        step={3600}
        value={startTime}
        onChange={(e) => setStartTime(e.target.value)}
      />{" "}
      이용시간:{" "}
      <input
        type="number"
        min={1}
        value={desiredHours}
        onChange={(e) => setDesiredHours(e.target.value)}
      />
      시간
      {timeFrom && hours > 0 ? (
        <>
          {" "}
          (종료:{" "}
          {new Date(
            timeFrom.getTime() + hours * 60 * 60 * 1000,
          ).toLocaleString()}
          )
        </>
      ) : null}
      <h3>옵션</h3>
      <AttachmentSelection
        attachments={attachments}
        unavailableAttachmentIds={unavailableAttachmentIds}
        selectedAttachmentIds={selectedAttachmentIds}
        onSelectAttachmentIds={setSelectedAttachmentIds}
      />
      <h3>결제</h3>
      <input
        type="checkbox"
        id="skip-payment"
        checked={skipPayment}
        onChange={(e) => setSkipPayment(e.target.checked)}
      />
      <label htmlFor="skip-payment">
        결제 생략 (즉시 확정 및 알림톡 발송)
      </label>
      {!skipPayment ? (
        <>
          <br />
          입금자명:{" "}
          <input
            type="text"
            placeholder="미입력시 예약자 이름"
            value={depositorName}
            onChange={(e) => setDepositorName(e.target.value)}
          />
          <br />
          금액:{" "}
          <input
            type="number"
            min={0}
            placeholder="미입력시 자동 계산"
            value={price}
            onChange={(e) => setPrice(e.target.value)}
          />
          원
          <br />
          <small>
            입금 대기 상태로 생성되며, 입금 확인 시 알림톡이 발송됩니다.
          </small>
        </>
      ) : null}
      <br />
      <br />
      <button className="primary" disabled={isSubmitting} onClick={submit}>
        예약 생성
      </button>
    </>
  );
}
