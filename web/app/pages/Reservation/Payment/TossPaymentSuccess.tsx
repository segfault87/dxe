import { isAxiosError } from "axios";
import React, { useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router";

import "./TossPayment.css";
import type { Route } from "./+types/TossPaymentSuccess";
import BookingService from "../../../api/booking";
import RequiresAuth from "../../../lib/RequiresAuth";
import type { BookingId } from "../../../types/models/base";
import { defaultErrorHandler } from "../../../lib/error";
import { redirect } from "react-router";

export function meta({}: Route.MetaArgs) {
  return [{ title: "결제 확인 | 드림하우스 합주실" }];
}

interface LoaderData {
  paymentType: string;
  orderId: string;
  paymentKey: string;
  amount: string;
  timeFrom: Date;
  timeTo: Date;
}

export async function clientLoader({
  request,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  const url = new URL(request.url);
  const searchParams = url.searchParams;

  const paymentType = searchParams.get("paymentType");
  const orderId = searchParams.get("orderId");
  const paymentKey = searchParams.get("paymentKey");
  const amount = searchParams.get("amount");

  if (!paymentType || !orderId || !paymentKey || !amount) {
    alert("잘못된 접근입니다.");
    throw redirect("/reservations/");
  }

  const state = await BookingService.getTossPaymentState(orderId);
  const timeFrom = new Date(state.data.timeFrom);
  const timeTo = new Date(state.data.timeTo);

  return {
    paymentType,
    orderId,
    paymentKey,
    amount,
    timeFrom,
    timeTo,
  };
}

function Error({ children }: { children: React.ReactNode }) {
  return (
    <div className="contents">
      <p>{children}</p>
      <Link to="/" className="cta">
        돌아가기
      </Link>
    </div>
  );
}

function TossPaymentSuccess({ loaderData }: { loaderData: LoaderData }) {
  const { paymentType, orderId, paymentKey, amount, timeFrom, timeTo } =
    loaderData;

  const navigate = useNavigate();

  const [bookingId, setBookingId] = useState<BookingId | null>(null);
  const [isRequestInProgress, setRequestInProgress] = useState(false);

  const cancelPayment = async () => {
    setRequestInProgress(true);

    try {
      await BookingService.cancelTossPayment(orderId);
      alert("취소되었습니다.");
      navigate("/reservation/");
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  const confirmPayment = async () => {
    setRequestInProgress(true);

    try {
      const result = await BookingService.confirmTossPayment({
        paymentKey,
        orderId,
        amount: parseInt(amount),
      });

      setBookingId(result.data.bookingId);
    } catch (error) {
      if (isAxiosError(error)) {
        const data = error.response?.data;
        if (data.message && data.extras.code) {
          navigate(
            `/reservation/payment/toss/fail/?orderId=${orderId}&code=${encodeURIComponent(data.extras.code)}&message=${encodeURIComponent(data.extras.message)}`,
          );
        } else {
          defaultErrorHandler(error);
        }
      } else {
        defaultErrorHandler(error);
      }
    } finally {
      setRequestInProgress(false);
    }
  };

  return (
    <div className="contents">
      {bookingId ? (
        <>
          <p>예약이 확정되었습니다. 이용해 주셔서 감사합니다.</p>
          <Link to={`/reservation/${bookingId}`} className="cta">
            예약 확인
          </Link>
        </>
      ) : (
        <>
          <p>
            아래 금액을 결제합니다.
            <br />
            확인 후 아래 확정 버튼을 눌러주세요.
          </p>
          <p className="date">
            이용 시간: {timeFrom.toLocaleString()} - {timeTo.toLocaleString()}
          </p>
          <p className="price">
            결제 금액: <em>₩{parseInt(amount).toLocaleString()}</em>
          </p>
          <span>
            <button
              className="cta"
              onClick={confirmPayment}
              disabled={isRequestInProgress}
            >
              예약 확정
            </button>{" "}
            <button
              className="cta"
              onClick={cancelPayment}
              disabled={isRequestInProgress}
            >
              취소
            </button>
          </span>
        </>
      )}
    </div>
  );
}

export default RequiresAuth(TossPaymentSuccess);
