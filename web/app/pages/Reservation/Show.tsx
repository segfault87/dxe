import { useState } from "react";
import { useNavigate, Link } from "react-router";

import "./Show.css";
import type { Route } from "./+types/Show";
import BookingService from "../../api/booking";
import { useAuth } from "../../context/AuthContext";
import RequiresAuth from "../../lib/RequiresAuth";
import type { Booking, CashPaymentStatus } from "../../types/models/booking";
import GroupSelection from "../../components/GroupSelection";
import Section from "../../components/Section";
import { defaultErrorHandler } from "../../lib/error";
import type { GroupWithUsers } from "../../types/models/group";

export function meta({}: Route.MetaArgs) {
  return [{ title: "예약 조회 | 드림하우스 합주실" }];
}

interface LoaderData {
  booking: Booking;
  cashPaymentStatus?: CashPaymentStatus;
}

export async function clientLoader({
  params,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  if (!params.bookingId) {
    throw new Error("bookingId is not supplied");
  }

  const result = await BookingService.get(params.bookingId);

  return {
    booking: result.data.booking,
    cashPaymentStatus: result.data.cashPaymentStatus,
  };
}

function ShowReservation({ loaderData }: Route.ComponentProps) {
  const auth = useAuth();
  const navigate = useNavigate();

  const { booking, cashPaymentStatus } = loaderData;
  const bookingStart = new Date(booking.bookingStart);
  const bookingEnd = new Date(booking.bookingEnd);

  const cancelReservation = async () => {
    let refundAccount: string | null = null;

    if (bookingStart.toDateString() !== new Date().toDateString()) {
      refundAccount = prompt("환불받으실 계좌번호를 입력해 주세요.");
      if (refundAccount === null) {
        return;
      }
    }

    try {
      await BookingService.cancel(booking.id, refundAccount);
    } catch (error) {
      defaultErrorHandler(error);
    }

    navigate(0);
  };

  const [groupTransferModal, setGroupTransferModal] = useState(false);

  const [isRequestInProgress, setRequestInProgress] = useState(false);

  const openDoor = async () => {
    setRequestInProgress(true);
    try {
      await BookingService.openDoor(booking.id);
      alert("문이 열렸습니다.");
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  const transferIdentity = async (group: GroupWithUsers) => {
    if (
      !confirm(
        `${group.name} 그룹으로 전환하시겠습니까? 이후 다시 변경할 수 없습니다.`,
      )
    ) {
      return;
    }

    try {
      await BookingService.amend(booking.id, {
        newIdentityId: group.id,
      });
      setGroupTransferModal(false);
      navigate(0);
    } catch (error) {
      defaultErrorHandler(error);
    }
  };

  let status: string;
  if (booking.status === "OVERDUE") {
    status = "만료";
  } else if (booking.status === "COMPLETE") {
    status = "이용완료";
  } else if (booking.status === "PENDING") {
    status = "미확정";
  } else if (booking.status === "CANCELED") {
    status = "취소";
  } else if (booking.confirmedAt !== null) {
    status = `확정 (일시: ${new Date(booking.confirmedAt).toLocaleString()})`;
  } else {
    status = "-";
  }

  return (
    <>
      {booking.status === "CONFIRMED" ||
      booking.status === "BUFFERED" ||
      booking.status === "IN_PROGRESS" ? (
        <div className="door-control">
          {booking.status !== "BUFFERED" && booking.status !== "IN_PROGRESS" ? (
            <p>출입구는 이용시간 30분 전부터 15분 이후까지 여실 수 있습니다.</p>
          ) : null}
          <button
            className="cta"
            onClick={openDoor}
            disabled={
              isRequestInProgress ||
              (booking.status !== "BUFFERED" &&
                booking.status !== "IN_PROGRESS")
            }
          >
            문 열기
          </button>
          <br />
          <p>
            문이 안 열리시면 <Link to="/inquiries">연락</Link> 바랍니다.
            <br />
            (엘리베이터 근처에 있는 음악연습실은 저희 업장이 아니며, 솔섬식품
            왼편에 있습니다.)
          </p>
        </div>
      ) : null}
      <Section id="booking-info" title="예약 정보">
        <ul>
          <li>
            예약 일시: {bookingStart.toLocaleString()} -{" "}
            {bookingEnd.toLocaleString()}
          </li>
          <li>
            예약자: {booking.customer.name}{" "}
            {booking.customer.type === "user" &&
            booking.status !== "OVERDUE" &&
            booking.status !== "COMPLETE" ? (
              <button
                style={{ marginLeft: "8px" }}
                className="primary"
                onClick={() => setGroupTransferModal(true)}
              >
                그룹으로 전환
              </button>
            ) : null}
          </li>
          <li>예약 상태: {status}</li>
        </ul>
      </Section>
      {cashPaymentStatus !== undefined ? (
        <Section id="payment-info" title="현금결제 정보">
          <ul>
            <li>금액 : ₩{cashPaymentStatus.price.toLocaleString()}</li>
            <li>
              확정 일시 :{" "}
              {booking.isConfirmed && cashPaymentStatus.confirmedAt !== null
                ? new Date(cashPaymentStatus.confirmedAt).toLocaleString()
                : "미확정"}
            </li>
            {cashPaymentStatus.isRefundRequested &&
            cashPaymentStatus.refundPrice !== null ? (
              <>
                <li>
                  환불 금액 : ₩{cashPaymentStatus.refundPrice.toLocaleString()}
                </li>
                <li>
                  환불 상태 :{" "}
                  {cashPaymentStatus.isRefunded &&
                  cashPaymentStatus.refundedAt !== null
                    ? `완료 (${new Date(cashPaymentStatus.refundedAt).toLocaleString()})`
                    : "미처리"}
                </li>
              </>
            ) : null}
            {cashPaymentStatus.refundAccount !== null ? (
              <li>
                환불 계좌 : {cashPaymentStatus.refundAccount.toLocaleString()}
              </li>
            ) : null}
          </ul>
          {booking.status === "PENDING" ? (
            <>
              <p>현금결제 시 아래 계좌로 입금해 주세요.</p>
              <ul>
                <li>계좌번호: 신한은행 110-609-686081</li>
                <li>계좌주: 박준규 (디엑스이 스튜디오)</li>
              </ul>
            </>
          ) : null}
        </Section>
      ) : null}
      {auth?.user.id === booking.holder.id &&
      (booking.status === "PENDING" || booking.status === "CONFIRMED") ? (
        <Section id="modify-reservation" title="예약 변경">
          <button className="primary" onClick={cancelReservation}>
            예약 취소
          </button>
        </Section>
      ) : null}
      <GroupSelection
        title="이전할 그룹 선택"
        bookingId={booking.id}
        onSelectGroup={transferIdentity}
        isOpen={groupTransferModal}
        close={() => {
          setGroupTransferModal(false);
        }}
      />
    </>
  );
}

export default RequiresAuth(ShowReservation);
