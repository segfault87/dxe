import { useCallback, useEffect, useState } from "react";
import { loadTossPayments } from "@tosspayments/tosspayments-sdk";
import type { TossPaymentsWidgets } from "@tosspayments/tosspayments-sdk";

import "./ExtendReservation.css";
import Modal from "./Modal";
import type { ModalProps } from "./Modal";
import BookingService from "../api/booking";
import { useEnv } from "../context/EnvContext";
import { DEFAULT_UNIT_ID } from "../constants";
import { defaultErrorHandler } from "../lib/error";
import type { Booking } from "../types/models/booking";
import type { UserId } from "../types/models/base";

export interface ExtendReservationProps extends ModalProps {
  userId: UserId;
  extendableHours: number;
  booking: Booking;
}

export default function ExtendReservation(props: ExtendReservationProps) {
  const { extendableHours, booking, userId } = props;

  const { tossPaymentClientKey } = useEnv();
  const [selectedHours, setSelectedHours] = useState(0);

  const [price, setPrice] = useState<number | null>(null);

  const [tossPaymentsWidgets, setTossPaymentsWidgets] =
    useState<TossPaymentsWidgets | null>(null);
  const [hasAgreedRequiredTerms, setAgreedRequiredTerms] = useState(true);

  const [isRequestInProgress, setRequestInProgress] = useState(false);

  const proceedPayment = async () => {
    setRequestInProgress(true);
    try {
      const result = await BookingService.amend(booking.id, {
        additionalHours: selectedHours,
      });

      if (result.data.foreignPaymentId) {
        await tossPaymentsWidgets?.requestPayment({
          orderId: result.data.foreignPaymentId,
          orderName: `추가 공간이용료 (${selectedHours} 시간)`,
          successUrl:
            window.location.origin + "/reservation/payment/toss/success/",
          failUrl: window.location.origin + "/reservation/payment/toss/fail/",
        });
      }
    } catch (error) {
      console.log(error);
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  const checkPrice = useCallback(
    async (hours: number) => {
      setRequestInProgress(true);
      try {
        const result = await BookingService.check({
          unitId: DEFAULT_UNIT_ID,
          timeFrom: booking.bookingStart,
          desiredHours: booking.bookingHours + hours,
          additionalHours: hours,
          excludeBookingId: booking.id,
        });

        setPrice(result.data.totalPrice);
        tossPaymentsWidgets?.setAmount({
          currency: "KRW",
          value: result.data.totalPrice,
        });
      } catch (error) {
        defaultErrorHandler(error);
      } finally {
        setRequestInProgress(false);
      }
    },
    [booking, tossPaymentsWidgets],
  );

  useEffect(() => {
    checkPrice(selectedHours);
  }, [checkPrice, selectedHours]);

  useEffect(() => {
    const initializeTossPayments = async () => {
      const tossPaymentsSdk = await loadTossPayments(tossPaymentClientKey);

      const customerKey = userId;
      const widgets = tossPaymentsSdk.widgets({
        customerKey,
      });

      await widgets.setAmount({
        currency: "KRW",
        value: 0,
      });

      try {
        await widgets.renderPaymentMethods({
          selector: "#toss-payment-method",
          variantKey: "DEFAULT",
        });
        const agreementWidget = await widgets.renderAgreement({
          selector: "#toss-payment-agreement",
          variantKey: "AGREEMENT",
        });

        agreementWidget.on("agreementStatusChange", (agreementStatus) => {
          setAgreedRequiredTerms(agreementStatus.agreedRequiredTerms);
        });

        setTossPaymentsWidgets(widgets);
      } catch {
        // suppress error
      }
    };

    initializeTossPayments();
  });

  const hours = [];
  for (let i = 1; i <= extendableHours; ++i) {
    hours.push(
      <span className="hour-selection" key={i}>
        <input
          type="radio"
          name="time"
          checked={selectedHours === i}
          onChange={() => setSelectedHours(i)}
          id={`time-${i}`}
        />
        <label htmlFor={`time-${i}`}>{i}시간</label>
      </span>,
    );
  }

  return (
    <Modal {...props}>
      <div className="extend-reservation">
        <div className="time-selection">
          <span className="label">연장 시간: </span>
          {hours}
        </div>
        <div className="payments">
          결제 금액: {price !== null ? <em>₩{price.toLocaleString()}</em> : "-"}
          <div id="toss-payment-method" />
          <div id="toss-payment-agreement" />
        </div>
      </div>
      <div className="actions">
        <button
          className="primary"
          onClick={proceedPayment}
          disabled={!price || isRequestInProgress || !hasAgreedRequiredTerms}
        >
          결제하기
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
