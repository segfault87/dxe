import { useMemo, useState } from "react";
import { useRevalidator } from "react-router";

import type { Route } from "./+types/AdHocPayments";
import AdminService from "../../api/admin";
import { defaultErrorHandler, loaderErrorHandler } from "../../lib/error";
import type { AdHocPaymentId } from "../../types/models/base";
import type { GroupWithUsers } from "../../types/models/group";
import type {
  AdHocPaymentStatus,
  AdHocPaymentWithUrl,
} from "../../types/models/payment";
import type { SelfUser } from "../../types/models/user";

interface LoaderData {
  payments: AdHocPaymentWithUrl[];
  users: SelfUser[];
  groups: GroupWithUsers[];
}

export async function clientLoader({
  request,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  try {
    const [payments, users, groups] = await Promise.all([
      AdminService.getAdHocPayments(),
      AdminService.getUsers(),
      AdminService.getGroups(),
    ]);

    return {
      payments: payments.data.payments,
      users: users.data.users,
      groups: groups.data.groups,
    };
  } catch (error) {
    throw loaderErrorHandler(error, request.url);
  }
}

const STATUS_LABELS: Record<AdHocPaymentStatus, string> = {
  PENDING: "결제 대기",
  PAID: "결제 완료",
  CANCELED: "취소됨",
};

const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text);
    alert("링크가 복사되었습니다.");
  } catch {
    prompt("아래 링크를 복사해 주세요.", text);
  }
};

export default function AdHocPayments({ loaderData }: Route.ComponentProps) {
  const { payments, users, groups } = loaderData;
  const revalidator = useRevalidator();

  const [recipientId, setRecipientId] = useState("");
  const [amount, setAmount] = useState("");
  const [note, setNote] = useState("");
  const [isSubmitting, setSubmitting] = useState(false);
  const [issuedPayment, setIssuedPayment] =
    useState<AdHocPaymentWithUrl | null>(null);

  const sortedUsers = useMemo(
    () => [...users].sort((a, b) => a.name.localeCompare(b.name)),
    [users],
  );
  const sortedGroups = useMemo(
    () => [...groups].sort((a, b) => a.name.localeCompare(b.name)),
    [groups],
  );

  const issue = async () => {
    const parsedAmount = parseInt(amount);

    if (!recipientId) {
      alert("수신자를 선택해 주세요.");
      return;
    }
    if (isNaN(parsedAmount) || parsedAmount <= 0) {
      alert("금액을 확인해 주세요.");
      return;
    }
    if (note.trim().length === 0) {
      alert("메모를 입력해 주세요.");
      return;
    }

    const recipient =
      users.find((v) => v.id === recipientId) ??
      groups.find((v) => v.id === recipientId);

    const message = [
      `수신자: ${recipient?.name ?? recipientId}`,
      `금액: ₩${parsedAmount.toLocaleString()}`,
      `메모: ${note.trim()}`,
      "",
      "결제 요청을 생성하시겠습니까?",
    ].join("\n");

    if (!confirm(message)) {
      return;
    }

    setSubmitting(true);
    try {
      const result = await AdminService.createAdHocPayment({
        amount: parsedAmount,
        recipientId,
        note: note.trim(),
      });

      setIssuedPayment(result.data.payment);
      setRecipientId("");
      setAmount("");
      setNote("");
      revalidator.revalidate();
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setSubmitting(false);
    }
  };

  const cancel = async (id: AdHocPaymentId) => {
    if (!confirm("결제 요청을 취소하시겠습니까?")) {
      return;
    }

    try {
      await AdminService.cancelAdHocPayment(id);
      revalidator.revalidate();
    } catch (error) {
      defaultErrorHandler(error);
    }
  };

  return (
    <>
      <h2>결제 요청 생성</h2>
      수신자:{" "}
      <select value={recipientId} onChange={(e) => setRecipientId(e.target.value)}>
        <option value="">선택</option>
        <optgroup label="개인">
          {sortedUsers.map((u) => (
            <option key={u.id} value={u.id}>
              {u.name}
            </option>
          ))}
        </optgroup>
        <optgroup label="그룹">
          {sortedGroups.map((g) => (
            <option key={g.id} value={g.id}>
              {g.name}
            </option>
          ))}
        </optgroup>
      </select>
      <br />
      금액:{" "}
      <input
        type="number"
        min={1}
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
      />
      원
      <br />
      메모:{" "}
      <input
        type="text"
        maxLength={200}
        value={note}
        onChange={(e) => setNote(e.target.value)}
        size={40}
      />
      <br />
      <button className="primary" disabled={isSubmitting} onClick={issue}>
        결제 요청 생성
      </button>
      {issuedPayment ? (
        <p>
          생성된 링크: <code>{issuedPayment.url}</code>{" "}
          <button onClick={() => copyToClipboard(issuedPayment.url)}>
            복사
          </button>
        </p>
      ) : null}
      <h2>결제 요청 목록</h2>
      <table>
        <thead>
          <tr>
            <th>생성일시</th>
            <th>수신자</th>
            <th>금액</th>
            <th>메모</th>
            <th>상태</th>
            <th>동작</th>
          </tr>
        </thead>
        <tbody>
          {payments.map((v) => (
            <tr key={v.id}>
              <td>{new Date(v.createdAt).toLocaleString()}</td>
              <td>
                {v.recipient.name}
                {v.recipient.type === "group" ? " (그룹)" : ""}
              </td>
              <td>₩{v.price.toLocaleString()}</td>
              <td>{v.note}</td>
              <td>
                {STATUS_LABELS[v.status]}
                {v.paidAt ? ` (${new Date(v.paidAt).toLocaleString()})` : ""}
              </td>
              <td>
                {v.status === "PENDING" ? (
                  <>
                    <button onClick={() => copyToClipboard(v.url)}>
                      링크 복사
                    </button>{" "}
                    <button className="primary" onClick={() => cancel(v.id)}>
                      취소
                    </button>
                  </>
                ) : null}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}
