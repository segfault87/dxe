import { useEffect } from "react";
import { useSearchParams } from "react-router";

import "./TossPayment.css";
import BookingService from "../../../api/booking";
import RequiresAuth from "../../../lib/RequiresAuth";
import { Link } from "react-router";

function TossPaymentFail() {
  const [searchParams, _] = useSearchParams();

  const orderId = searchParams.get("orderId");
  const code = searchParams.get("code");
  const message = searchParams.get("message");

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
