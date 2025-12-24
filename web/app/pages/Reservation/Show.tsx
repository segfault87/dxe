import { useState } from "react";
import ReactGA from "react-ga4";
import { useLocation, useNavigate, Link } from "react-router";

import "./Show.css";
import type { Route } from "./+types/Show";
import BookingService from "../../api/booking";
import UserService from "../../api/user";
import { useAuth } from "../../context/AuthContext";
import RequiresAuth from "../../lib/RequiresAuth";
import type { Booking } from "../../types/models/booking";
import type { Transaction } from "../../types/models/payment";
import ExtendReservation from "../../components/ExtendReservation";
import GroupInvitationModal from "../../components/GroupInvitation";
import GroupSelectionModal from "../../components/GroupSelection";
import LocationInformation from "../../components/LocationInformation";
import Section from "../../components/Section";
import { defaultErrorHandler } from "../../lib/error";
import type { GroupWithUsers } from "../../types/models/group";

export function meta({}: Route.MetaArgs) {
  return [{ title: "예약 조회 | 드림하우스 합주실" }];
}

interface LoaderData {
  booking: Booking | null;
  transaction: Transaction | null;
  amendable: boolean;
  extendableHours: number;
  group: GroupWithUsers | null;
}

export async function clientLoader({
  params,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  if (!params.bookingId) {
    throw new Error("bookingId is not supplied");
  }

  try {
    const bookingResult = await BookingService.get(params.bookingId);
    let group: GroupWithUsers | null;
    if (bookingResult.data.booking.customer.type === "group") {
      const groupResult = await UserService.getGroup(
        bookingResult.data.booking.customer.id,
      );
      group = groupResult.data.group;
    } else {
      group = null;
    }

    return {
      booking: bookingResult.data.booking,
      transaction: bookingResult.data.transaction,
      amendable: bookingResult.data.amendable,
      extendableHours: bookingResult.data.extendableHours,
      group: group,
    };
  } catch (error) {
    defaultErrorHandler(error);
    return {
      booking: null,
      transaction: null,
      amendable: false,
      extendableHours: 0,
      group: null,
    };
  }
}

function ShowReservationInner({
  booking,
  transaction,
  amendable,
  extendableHours,
  group,
}: {
  booking: Booking;
  transaction: Transaction | null;
  amendable: boolean;
  extendableHours: number;
  group: GroupWithUsers | null;
}) {
  const auth = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  const bookingStart = new Date(booking.bookingStart);
  const bookingEnd = new Date(booking.bookingEnd);

  const [currentGroup, setGroup] = useState<GroupWithUsers | null>(group);
  const [groupInvitationModal, setGroupInvitationModal] = useState(false);

  const cancelReservation = async () => {
    let refundAccount: string | null = null;

    if (transaction?.cash) {
      if (bookingStart.toDateString() !== new Date().toDateString()) {
        refundAccount = prompt(
          "환불받으실 계좌번호를 입력해 주세요.",
          auth?.user.refundAccount ?? undefined,
        );
        if (refundAccount === null) {
          return;
        }
      }

      try {
        await BookingService.cancel(booking.id, refundAccount);
      } catch (error) {
        defaultErrorHandler(error);
      }
    } else if (transaction?.tossPayments) {
      const cancelReason = prompt("취소 사유를 입력해 주세요.");
      if (cancelReason === null) {
        return;
      }

      try {
        await BookingService.cancel(
          booking.id,
          null,
          cancelReason ? cancelReason : null,
        );
      } catch (error) {
        defaultErrorHandler(error);
      }
    }

    navigate(0);
  };

  const [groupTransferModal, setGroupTransferModal] = useState(false);
  const [extendReservationModal, setExtendReservationModal] = useState(false);

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
      ReactGA.event("group_transfer");
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
            문이 열리지 않는다면 <Link to="/inquiries/">연락</Link> 바랍니다.
            {booking.customer.type === "group" &&
            currentGroup?.isOpen === true ? (
              <>
                <br />이 페이지를 다른 멤버들에게 공유하시려면 아래{" "}
                <em>초대</em> 버튼으로 멤버들을 초대해 주세요.
              </>
            ) : null}
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
            {booking.status !== "OVERDUE" && booking.status !== "COMPLETE" ? (
              <>
                {booking.customer.type === "user" ? (
                  <button
                    style={{ marginLeft: "8px" }}
                    className="primary"
                    onClick={() => {
                      ReactGA.event("click_group_transfer");
                      setGroupTransferModal(true);
                    }}
                  >
                    그룹으로 전환
                  </button>
                ) : null}
                {booking.customer.type === "group" &&
                currentGroup?.id === booking.customer.id &&
                currentGroup?.isOpen === true ? (
                  <button
                    style={{ marginLeft: "8px" }}
                    className="primary"
                    onClick={() => {
                      ReactGA.event("click_group_invitation");
                      setGroupInvitationModal(true);
                    }}
                  >
                    초대
                  </button>
                ) : null}
              </>
            ) : null}
          </li>
          <li>예약 상태: {status}</li>
        </ul>
      </Section>
      {transaction?.cash ? (
        <Section id="payment-info" title="현금결제 정보">
          <ul>
            <li>금액 : ₩{transaction.cash.price.toLocaleString()}</li>
            <li>
              확정 일시 :{" "}
              {booking.isConfirmed && transaction.cash.confirmedAt !== null
                ? new Date(transaction.cash.confirmedAt).toLocaleString()
                : "미확정"}
            </li>
            {transaction.cash.isRefundRequested &&
            transaction.cash.refundPrice !== null ? (
              <>
                <li>
                  환불 금액 : ₩{transaction.cash.refundPrice.toLocaleString()}
                </li>
                <li>
                  환불 상태 :{" "}
                  {transaction.cash.isRefunded &&
                  transaction.cash.refundedAt !== null
                    ? `완료 (${new Date(transaction.cash.refundedAt).toLocaleString()})`
                    : "미처리"}
                </li>
              </>
            ) : null}
            {transaction.cash.refundAccount !== null ? (
              <li>
                환불 계좌 : {transaction.cash.refundAccount.toLocaleString()}
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
      <Section id="location" title="오시는 길">
        <LocationInformation buttonClassName="primary" />
      </Section>
      {transaction?.tossPayments ? (
        <Section id="payment-info" title="결제 정보">
          <ul>
            <li>금액 : ₩{transaction.tossPayments.price.toLocaleString()}</li>
            <li>
              확정 일시 :{" "}
              {booking.isConfirmed &&
              transaction.tossPayments.confirmedAt !== null
                ? new Date(
                    transaction.tossPayments.confirmedAt,
                  ).toLocaleString()
                : "미확정"}
            </li>
            {transaction.tossPayments.refundPrice !== null ? (
              <>
                <li>
                  환불 금액 : ₩
                  {transaction.tossPayments.refundPrice.toLocaleString()}
                </li>
                <li>
                  환불 상태 :{" "}
                  {transaction.tossPayments.isRefunded &&
                  transaction.tossPayments.refundedAt !== null
                    ? `완료 (${new Date(transaction.tossPayments.refundedAt).toLocaleString()})`
                    : "미처리"}
                </li>
              </>
            ) : null}
          </ul>
        </Section>
      ) : null}
      {auth?.user.id === booking.holder.id &&
      (booking.status === "PENDING" || booking.status === "CONFIRMED") ? (
        <Section id="modify-reservation" title="예약 변경">
          <button className="primary" onClick={cancelReservation}>
            예약 취소
          </button>
          {amendable ? (
            <>
              {" "}
              <Link to={`/reservation/${booking.id}/amend`} className="primary">
                예약 변경
              </Link>
            </>
          ) : null}
          {extendableHours > 0 ? (
            <>
              {" "}
              <button
                className="primary"
                onClick={() => setExtendReservationModal(true)}
              >
                예약 연장
              </button>
            </>
          ) : null}
        </Section>
      ) : null}
      {currentGroup ? (
        <GroupInvitationModal
          group={currentGroup}
          redirectTo={location.pathname}
          isOpen={groupInvitationModal}
          close={() => setGroupInvitationModal(false)}
        />
      ) : null}
      <GroupSelectionModal
        title="이전할 그룹 선택"
        bookingId={booking.id}
        onSelectGroup={(group) => {
          setGroup(group);
          transferIdentity(group);
        }}
        isOpen={groupTransferModal}
        close={() => setGroupTransferModal(false)}
      />
      {auth ? (
        <ExtendReservation
          userId={auth.user.id}
          booking={booking}
          extendableHours={extendableHours}
          isOpen={extendReservationModal}
          close={() => setExtendReservationModal(false)}
        />
      ) : null}
    </>
  );
}

function ShowReservation({ loaderData }: Route.ComponentProps) {
  if (loaderData.booking) {
    return (
      <ShowReservationInner
        booking={loaderData.booking}
        transaction={loaderData.transaction}
        amendable={loaderData.amendable}
        extendableHours={loaderData.extendableHours}
        group={loaderData.group}
      />
    );
  } else {
    return <></>;
  }
}

export default RequiresAuth(ShowReservation);
