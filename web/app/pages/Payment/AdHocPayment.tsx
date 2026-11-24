import { useEffect, useState } from "react";
import { loadTossPayments } from "@tosspayments/tosspayments-sdk";
import type { TossPaymentsWidgets } from "@tosspayments/tosspayments-sdk";
import { Link, redirect } from "react-router";

import "../Reservation/Payment/TossPayment.css";
import type { Route } from "./+types/AdHocPayment";
import BookingService from "../../api/booking";
import { useEnv } from "../../context/EnvContext";
import RequiresAuth from "../../lib/RequiresAuth";
import type { AuthProps } from "../../lib/RequiresAuth";
import { defaultErrorHandler, loaderErrorHandler } from "../../lib/error";
import type { AdHocPayment as AdHocPaymentModel } from "../../types/models/payment";

// Toss Payments limits orderName to 100 characters
const MAX_ORDER_NAME_LENGTH = 100;

export function meta(): Route.MetaDescriptors {
  return [{ title: "결제 요청 | 드림하우스 합주실" }];
}

interface LoaderData {
  payment: AdHocPaymentModel;
}

export async function clientLoader({
  request,
  params,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  if (!params.adHocPaymentId) {
    throw redirect("/");
  }

  try {
    const result = await BookingService.getAdHocPayment(params.adHocPaymentId);

    return { payment: result.data.payment };
  } catch (error) {
    throw loaderErrorHandler(error, request.url);
  }
}

function AdHocPayment({ loaderData, auth }: Route.ComponentProps & AuthProps) {
  const { payment } = loaderData;
  const { tossPaymentClientKey } = useEnv();

  const [tossPaymentsWidgets, setTossPaymentsWidgets] =
    useState<TossPaymentsWidgets | null>(null);
  const [hasAgreedRequiredTerms, setAgreedRequiredTerms] = useState(true);
  const [isRequestInProgress, setRequestInProgress] = useState(false);

  const isPayable = payment.status === "PENDING";
  const userId = auth.user.id;

  useEffect(() => {
    if (!isPayable) {
      return;
    }

    const initializeTossPayments = async () => {
      const tossPaymentsSdk = await loadTossPayments(tossPaymentClientKey);
      const widgets = tossPaymentsSdk.widgets({ customerKey: userId });

      await widgets.setAmount({ currency: "KRW", value: payment.price });

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
  }, [isPayable, payment.price, tossPaymentClientKey, userId]);

  const proceedPayment = async () => {
    if (!tossPaymentsWidgets) {
      return;
    }

    setRequestInProgress(true);
    try {
      const result = await BookingService.initiateAdHocTossPayment(payment.id);

      await tossPaymentsWidgets.requestPayment({
        orderId: result.data.orderId,
        orderName: payment.note.slice(0, MAX_ORDER_NAME_LENGTH),
        successUrl:
          window.location.origin + "/reservation/payment/toss/success/",
        failUrl: window.location.origin + "/reservation/payment/toss/fail/",
      });
    } catch (error) {
      defaultErrorHandler(error);
    } finally {
      setRequestInProgress(false);
    }
  };

  if (!isPayable) {
    return (
      <div className="contents">
        <p>
          {payment.status === "PAID"
            ? "이미 결제가 완료된 요청입니다."
            : "취소된 결제 요청입니다."}
        </p>
        <p>{payment.note}</p>
        <Link to="/" className="cta">
          돌아가기
        </Link>
      </div>
    );
  }

  return (
    <div className="contents">
      <p>
        {payment.recipient.name}님께 아래 금액의 결제가 요청되었습니다.
      </p>
      <p>{payment.note}</p>
      <p className="price">
        결제 금액: <em>₩{payment.price.toLocaleString()}</em>
      </p>
      <div style={{ width: "100%" }}>
        <div id="toss-payment-method" />
        <div id="toss-payment-agreement" />
      </div>
      <button
        className="cta"
        onClick={proceedPayment}
        disabled={
          !tossPaymentsWidgets ||
          !hasAgreedRequiredTerms ||
          isRequestInProgress
        }
      >
        결제하기
      </button>
    </div>
  );
}

export default RequiresAuth(AdHocPayment);
