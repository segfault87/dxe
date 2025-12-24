import { useEffect } from "react";
import ReactGA from "react-ga4";

import "./TossPayment.css";
import type { Route } from "./+types/TossPaymentFail";
import BookingService from "../../../api/booking";
import RequiresAuth from "../../../lib/RequiresAuth";
import { Link } from "react-router";

interface LoaderData {
  orderId: string | null;
  code: string | null;
  message: string | null;
}

export async function clientLoader({
  request,
}: Route.ClientLoaderArgs): Promise<LoaderData> {
  const url = new URL(request.url);
  const searchParams = url.searchParams;

  const orderId = searchParams.get("orderId");
  const code = searchParams.get("code");
  const message = searchParams.get("message");

  return { orderId, code, message };
}

function TossPaymentFail({ loaderData }: Route.ComponentProps) {
  const { orderId, code, message } = loaderData;

  useEffect(() => {
    if (code && message) {
      ReactGA.event("payment_failure", { code: code, message: message });
    }
  }, [code, message]);

  useEffect(() => {
    const cancel = async () => {
      if (orderId) {
        await BookingService.cancelTossPayment(orderId);
      }
    };

    cancel();
  }, [orderId]);

  return (
    <div id="contents">
      <p>결제에 실패했습니다.</p>
      <p>
        {message} ({code})
      </p>
      <Link to="/" className="cta">
        돌아가기
      </Link>
    </div>
  );
}

export default RequiresAuth(TossPaymentFail);
